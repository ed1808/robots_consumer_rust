use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

pub struct RobotConsumer {
    pub dirname: String,
    pub filename: String,
    dirpath: Option<PathBuf>,
    consumer_config: HashMap<String, Vec<String>>,
    error_detail: String,
}

impl RobotConsumer {
    pub fn new(robot_dirname: String, robot_filename: String) -> RobotConsumer {
        let dirname = robot_dirname;
        let filename = robot_filename;

        return RobotConsumer {
            dirname,
            filename,
            dirpath: None,
            consumer_config: HashMap::new(),
            error_detail: String::new(),
        };
    }

    pub fn start(&mut self) {
        match self.init() {
            Ok(()) => {
                println!("[CONSUMER MESSAGE] Iniciando proceso");

                let mut delay: u64 = 1;

                if let Some(delay_string) = self.consumer_config.get("delay") {
                    if let Some(first_delay_string) = delay_string.first() {
                        if let Ok(delay_number) = first_delay_string.parse::<u64>() {
                            delay = delay_number;
                        } else {
                            println!("No se pudo convertir a número: {}", first_delay_string);
                        }
                    } else {
                        println!("La clave 'delay' no contiene ningún valor");
                    }
                } else {
                    panic!("No se especificó un delay {:?}", self.consumer_config)
                }

                loop {
                    for url in self.consumer_config.get("robotsUrls").unwrap() {
                        println!("[CONSUMER MESSAGE] Consultando url {}", url);

                        match reqwest::blocking::get(url) {
                            Ok(response) => {
                                if response.status().is_server_error() {
                                    self.error_detail =
                                        String::from(response.status().canonical_reason().unwrap());
                                } else if response.status().is_client_error() {
                                    self.error_detail =
                                        String::from(response.status().canonical_reason().unwrap());
                                } else {
                                    self.error_detail.clear();
                                }

                                if !self.error_detail.is_empty() {
                                    println!(
                                        "[CONSUMER MESSAGE] Ha ocurrido un error en la url {}",
                                        url
                                    );
                                    println!("Código: {}", response.status().as_u16());
                                    println!("Error: {}", response.text().unwrap_err());
                                }
                            }
                            Err(e) => {
                                println!(
                                    "[CONSUMER MESSAGE] Ha ocurrido el siguiente error: {}",
                                    e
                                );
                            }
                        }

                        println!("############################################");

                        sleep(Duration::from_secs(delay));
                    }
                }
            }
            Err(error) => {
                println!("Ha ocurrido el siguiente error: {}", error);
            }
        }
    }

    fn get_dirpath(&self) -> Option<PathBuf> {
        println!("[CONSUMMER MESSAGE] Buscando directorio {}", self.dirname);

        let documents_dir = dirs::document_dir().unwrap();
        let searched_dirname = OsStr::new(&self.dirname);

        if env::set_current_dir(documents_dir).is_ok() {
            let current_path = env::current_dir().unwrap();

            if let Ok(entries) = fs::read_dir(current_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        if path.is_dir() {
                            if let Some(dirname) = path.file_name() {
                                if dirname == searched_dirname {
                                    return Some(env::current_dir().unwrap().join(dirname));
                                }
                            }
                        }
                    }
                }
            }
        }

        return None;
    }

    fn init(&mut self) -> Result<(), String> {
        match self.get_dirpath() {
            Some(dirpath) => {
                self.dirpath = Some(dirpath.clone());

                if !self.file_in_dir() {
                    return Err(format!(
                        "El archivo {} no existe en el directorio actual",
                        self.filename
                    ));
                }

                let file_content = fs::read_to_string(dirpath.join(&self.filename))
                    .map_err(|_| "No se pudo leer el archivo")?;

                self.consumer_config =
                    serde_json::from_str(&file_content).map_err(|e| format!("{}", e))?;

                return Ok(());
            }
            None => Err(String::from("No se encontró el directorio")),
        }
    }

    fn file_in_dir(&self) -> bool {
        let searched_filename = OsStr::new(&self.filename);
        if let Some(dirpath) = &self.dirpath {
            if let Ok(entries) = dirpath.read_dir() {
                entries
                    .filter_map(|entry| entry.ok())
                    .any(|entry| entry.file_name().to_os_string() == searched_filename)
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
}
