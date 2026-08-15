fn main() {
    if std::env::var("BLUELINK_PRODUCTION").as_deref() == Ok("1") {
        let database = std::path::Path::new("../data/production/articles.sqlite");
        let marker = std::path::Path::new("../data/production/PRODUCTION_DATASET");
        if !database.exists() || !marker.exists() {
            panic!("BLUELINK_PRODUCTION=1 requires data/production/articles.sqlite and its PRODUCTION_DATASET marker. Refusing to package the development dataset.");
        }
    }
    tauri_build::build()
}
