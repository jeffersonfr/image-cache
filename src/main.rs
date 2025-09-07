use actix_web::{get, App, HttpResponse, HttpServer, Error, web};
use actix_web::http::StatusCode;
use redis::{Client, AsyncCommands};
use std::path::{Path};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::{File, Metadata};
use std::io::Read;

const IMAGE_FOLDER: &str = "images";

struct AppState {
    redis_client: Arc<Mutex<Client>>,
}

async fn get_cached_image(client: Arc<Mutex<Client>>, filename: &Path) -> Result<Option<Vec<u8>>, redis::RedisError> {
    let client = client.lock().await;
    let mut conn = client.get_async_connection().await?;

    conn.get(filename.to_str()).await
}

async fn cache_image(client: Arc<Mutex<Client>>, filename: &Path, data: &[u8]) -> Result<(), redis::RedisError> {
    let client = client.lock().await;
    let mut conn = client.get_async_connection().await?;

    conn.set(filename.to_str(), data).await
}

fn get_file_as_byte_vec(filename: &Path) -> Option<Vec<u8>> {
    let basepath = Path::new(&IMAGE_FOLDER);
    let fullpath = basepath.join(&filename);

    if !fullpath.exists() {
        return None;
    }

    let mut file: File;
    
    if let Ok(data) = File::open(&fullpath) {
        file = data;
    } else {
        return None;
    }

    let metadata: Metadata;
    
    if let Ok(data) = std::fs::metadata(&fullpath) {
        metadata = data;
    } else {
        return None;
    }

    let mut buffer = vec![0; metadata.len() as usize];
    
    file.read(&mut buffer).expect("buffer overflow");

    Some(buffer)
}

fn is_valid_folder_name(name: &str) -> bool {
    !name.contains('%') && 
    !name.contains('\\') && 
    !name.contains("..") && 
    !name.is_empty() &&
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

#[get("/image/{directory}/{filename}")]
async fn get_image(params: web::Path<(String, String)>, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let directory = params.0.as_str();
    let filename = params.1.as_str();

    if !is_valid_folder_name(&directory) {
        return Err(actix_web::error::ErrorBadRequest("Invalid access"));
    }

    if !is_valid_folder_name(&filename) {
        return Err(actix_web::error::ErrorBadRequest("Invalid access"));
    }

    let basepath = Path::new(params.0.as_str());
    let fullpath = basepath.join(&params.1.as_str());

    let client = state.redis_client.clone();
    
    // Verificar o cache no Redis
    match get_cached_image(client.clone(), &fullpath).await {
        Ok(Some(image_data)) => {
            return Ok(HttpResponse::Ok()
                .content_type("image/jpeg")
                .body(image_data));
        }
        Err(e) => eprintln!("Unable to connect to redis: {}", e),
        _ => (),
    }

    // Carregar do sistema de arquivos
    let bytes: Vec<u8>;

    if let Some(data) = get_file_as_byte_vec(&fullpath) {
        bytes = data;
    } else {
        return Ok(HttpResponse::build(StatusCode::NOT_FOUND).finish());
    }

    // Armazenar no cache (em segundo plano)
    let client_clone = client.clone();
    let filename_clone = fullpath.clone();
    let bytes_clone = bytes.clone();

    tokio::spawn(async move {
        match cache_image(client_clone, &filename_clone, &bytes_clone).await {
            Err(e) => eprint!("Unable to cache image: {}", &e),
            _ => ()
        }
    });

    Ok(HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(bytes))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Criar pasta de imagens se não existir
    tokio::fs::create_dir_all(IMAGE_FOLDER).await?;

    // Criar cliente Redis
    let redis_client = Arc::new(Mutex::new(
        Client::open("redis://redis/").expect("Failed to connect to Redis")
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                redis_client: redis_client.clone(),
            }))
            .service(get_image)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
