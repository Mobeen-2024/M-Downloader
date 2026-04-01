fn main() { let text = reqwest::blocking::get("https://youtu.be/y2iMrxN4NyE?si=DQdmh6grNxgetZwq").unwrap().text().unwrap(); std::fs::write("yt2.html", text).unwrap(); }
