use boa_engine::{Context, Source, JsValue, JsString};
use reqwest::Client;
use tokio::sync::Mutex;
use regex::Regex;

pub struct YouTubeDeobfuscator {
    client: Client,
    // Caches for the current session's functions to avoid re-parsing
    n_func: Mutex<Option<String>>,
    sig_func: Mutex<Option<String>>,
}

impl YouTubeDeobfuscator {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: Client::new(),
            n_func: Mutex::new(None),
            sig_func: Mutex::new(None),
        })
    }

    /// Solves the YouTube "n" parameter by executing the extracted player JS logic via boa_engine.
    pub async fn solve_n(&self, n: &str, base_js_url: &str) -> Result<String, String> {
        let mut cached_n = self.n_func.lock().await;
        
        if cached_n.is_none() {
            log::info!("[Deobfuscator] Extracting 'n' solver from {}", base_js_url);
            let js_content = self.client.get(base_js_url).send().await
                .map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
            
            let n_regex = Regex::new(r#"(?:\.get\("n"\)\)&&\(b=([a-zA-Z0-9$]+)(?:\[(\d+)\])?\([a-zA-Z0-9$]+\))"#).unwrap();
            if let Some(caps) = n_regex.captures(&js_content) {
                let func_name = &caps[1];
                log::info!("[Deobfuscator] Identified 'n' function: {}", func_name);
                
                let body_regex = Regex::new(&format!(r#"{}\s*=\s*function\s*\(\s*a\s*\)\s*\{{([\s\S]+?)\}};"#, regex::escape(func_name))).unwrap();
                if let Some(body_caps) = body_regex.captures(&js_content) {
                    *cached_n = Some(format!("function solve_n(a) {{ {} }}", &body_caps[1]));
                }
            }
        }

        if let Some(ref js_func) = *cached_n {
            // Pure Rust execution sandbox (boa_engine)
            let mut ctx = Context::default();
            ctx.eval(Source::from_bytes(js_func.as_bytes())).map_err(|e| e.to_string())?;
            
            // Invoke the solved function (v0.20 requires JsString keys)
            let solve_n_fn = ctx.global_object().get(JsString::from("solve_n"), &mut ctx).map_err(|e| e.to_string())?;
            if let Some(func) = solve_n_fn.as_object() {
                let n_js = JsValue::from(JsString::from(n));
                let result = func.call(&JsValue::undefined(), &[n_js], &mut ctx)
                    .map_err(|e| e.to_string())?;
                Ok(result.as_string().map(|s| s.to_std_string_escaped()).unwrap_or_default())
            } else {
                Err("solve_n is not a function in the context".to_string())
            }
        } else {
            Err("Failed to extract 'n' solver function".to_string())
        }
    }

    /// Solves the YouTube signature by executing the transformation logic via boa_engine.
    pub async fn solve_signature(&self, s: &str, base_js_url: &str) -> Result<String, String> {
        let mut cached_sig = self.sig_func.lock().await;

        if cached_sig.is_none() {
            log::info!("[Deobfuscator] Extracting signature solver from {}", base_js_url);
            let js_content = self.client.get(base_js_url).send().await
                .map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;

            let sig_regex = Regex::new(r#"([a-zA-Z0-9$]+)\s*=\s*function\(\s*a\s*\)\s*\{\s*a\s*=\s*a\.split\(\s*""\s*\);\s*([a-zA-Z0-9$]+)\."#).unwrap();
            if let Some(caps) = sig_regex.captures(&js_content) {
                let func_name = &caps[1];
                let obj_name = &caps[2];
                log::info!("[Deobfuscator] Identified signature function: {} (Obj: {})", func_name, obj_name);

                let obj_regex = Regex::new(&format!(r#"var\s+{}\s*=\s*\{{[\s\S]+?\}};"#, regex::escape(obj_name))).unwrap();
                let body_regex = Regex::new(&format!(r#"{}\s*=\s*function\s*\(\s*a\s*\)\s*\{{([\s\S]+?)\}};"#, regex::escape(func_name))).unwrap();

                if let (Some(o_caps), Some(b_caps)) = (obj_regex.captures(&js_content), body_regex.captures(&js_content)) {
                    *cached_sig = Some(format!("{}; function solve_sig(a) {{ {} }}", &o_caps[0], &b_caps[0]));
                }
            }
        }

        if let Some(ref js_func) = *cached_sig {
            let mut ctx = Context::default();
            ctx.eval(Source::from_bytes(js_func.as_bytes())).map_err(|e| e.to_string())?;
            
            let solve_sig_fn = ctx.global_object().get(JsString::from("solve_sig"), &mut ctx).map_err(|e| e.to_string())?;
            if let Some(func) = solve_sig_fn.as_object() {
                let s_js = JsValue::from(JsString::from(s));
                let result = func.call(&JsValue::undefined(), &[s_js], &mut ctx)
                    .map_err(|e| e.to_string())?;
                Ok(result.as_string().map(|s| s.to_std_string_escaped()).unwrap_or_default())
            } else {
                Err("solve_sig is not a function in the context".to_string())
            }
        } else {
            Err("Failed to extract signature solver function".to_string())
        }
    }
}
