use rquickjs::{Context, Runtime};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use regex::Regex;

pub struct YouTubeDeobfuscator {
    client: Client,
    runtime: Runtime,
    context: Context,
    // Caches for the current session's functions to avoid re-parsing
    n_func: Mutex<Option<String>>,
    sig_func: Mutex<Option<String>>,
    base_js_url: Mutex<Option<String>>,
}

impl YouTubeDeobfuscator {
    pub fn new() -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;
        
        Ok(Self {
            client: Client::new(),
            runtime,
            context,
            n_func: Mutex::new(None),
            sig_func: Mutex::new(None),
            base_js_url: Mutex::new(None),
        })
    }

    /// Solves the YouTube "n" parameter by executing the extracted player JS logic.
    pub async fn solve_n(&self, n: &str, base_js_url: &str) -> Result<String, String> {
        let mut cached_n = self.n_func.lock().await;
        
        if cached_n.is_none() {
            log::info!("[Deobfuscator] Extracting 'n' solver from {}", base_js_url);
            let js_content = self.client.get(base_js_url).send().await
                .map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;
            
            // Refined regex for the "n" parameter transformation function (inspired by yt-dlp)
            let n_regex = Regex::new(r#"(?:\.get\("n"\)\)&&\(b=([a-zA-Z0-9$]+)(?:\[(\d+)\])?\([a-zA-Z0-9$]+\))"#).unwrap();
            if let Some(caps) = n_regex.captures(&js_content) {
                let func_name = &caps[1];
                log::info!("[Deobfuscator] Identified 'n' function: {}", func_name);
                
                // Extract the function body
                let body_regex = Regex::new(&format!(r#"{}\s*=\s*function\s*\(\s*a\s*\)\s*\{{([\s\S]+?)\}};"#, regex::escape(func_name))).unwrap();
                if let Some(body_caps) = body_regex.captures(&js_content) {
                    *cached_n = Some(format!("function solve_n(a) {{ {} }}", &body_caps[1]));
                }
            }
        }

        if let Some(ref js_func) = *cached_n {
            // Execute in QuickJS sandbox
            self.context.with(|ctx| {
                let _ = ctx.eval::<(), _>(js_func);
                let func: rquickjs::Function = ctx.globals().get("solve_n").unwrap();
                let result: String = func.call((n,)).unwrap();
                Ok(result)
            })
        } else {
            Err("Failed to extract 'n' solver function".to_string())
        }
    }

    /// Solves the YouTube signature by executing the transformation logic.
    pub async fn solve_signature(&self, s: &str, base_js_url: &str) -> Result<String, String> {
        let mut cached_sig = self.sig_func.lock().await;

        if cached_sig.is_none() {
            log::info!("[Deobfuscator] Extracting signature solver from {}", base_js_url);
            let js_content = self.client.get(base_js_url).send().await
                .map_err(|e| e.to_string())?.text().await.map_err(|e| e.to_string())?;

            // Regex for signature deobfuscation function
            let sig_regex = Regex::new(r#"([a-zA-Z0-9$]+)\s*=\s*function\(\s*a\s*\)\s*\{\s*a\s*=\s*a\.split\(\s*""\s*\);\s*([a-zA-Z0-9$]+)\."#).unwrap();
            if let Some(caps) = sig_regex.captures(&js_content) {
                let func_name = &caps[1];
                let obj_name = &caps[2];
                log::info!("[Deobfuscator] Identified signature function: {} (Obj: {})", func_name, obj_name);

                // Extract the function AND the helper object
                let obj_regex = Regex::new(&format!(r#"var\s+{}\s*=\s*\{{[\s\S]+?\}};"#, regex::escape(obj_name))).unwrap();
                let body_regex = Regex::new(&format!(r#"{}\s*=\s*function\s*\(\s*a\s*\)\s*\{{([\s\S]+?)\}};"#, regex::escape(func_name))).unwrap();

                if let (Some(o_caps), Some(b_caps)) = (obj_regex.captures(&js_content), body_regex.captures(&js_content)) {
                    *cached_sig = Some(format!("{}; function solve_sig(a) {{ {} }}", &o_caps[0], &b_caps[0]));
                }
            }
        }

        if let Some(ref js_func) = *cached_sig {
            self.context.with(|ctx| {
                let _ = ctx.eval::<(), _>(js_func);
                let func: rquickjs::Function = ctx.globals().get("solve_sig").unwrap();
                let result: String = func.call((s,)).unwrap();
                Ok(result)
            })
        } else {
            Err("Failed to extract signature solver function".to_string())
        }
    }
}
