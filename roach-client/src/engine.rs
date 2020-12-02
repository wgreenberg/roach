use async_trait::async_trait;
use hive::engine::Engine;
use crate::process;

pub fn get_engine(ai_path: String, engine_type: EngineType) -> Box<dyn UHPCompliant> {
    match engine_type {
        EngineType::UHP => Box::new(UHPEngine::new(ai_path)),
        EngineType::Simple => Box::new(SimpleEngine::new(ai_path)),
    }
}

pub enum EngineType {
    UHP,
    Simple,
}

#[async_trait]
pub trait UHPCompliant {
    async fn handle_command(&mut self, input: &str) -> String;
}

pub struct SimpleEngine {
    process: process::Process,
    real_engine: Engine,
}

impl SimpleEngine {
    pub fn new(ai_path: String) -> Self {
        let real_engine = Engine::new();
        let process = process::Process::new(&ai_path);
        SimpleEngine { process, real_engine }
    }
}

#[async_trait]
impl UHPCompliant for SimpleEngine {
    async fn handle_command(&mut self, input: &str) -> String {
        if input == "bestmove" {
            if let Some(game) = &self.real_engine.game {
                let game_state = format!("{}", game);
                self.process.send(&game_state, false).await
            } else {
                panic!("game not initialized yet!");
            }
        } else {
            self.real_engine.handle_command(input)
        }
    }
}

pub struct UHPEngine {
    process: process::Process,
}

impl UHPEngine {
    fn new(ai_path: String) -> Self {
        let process = process::Process::new(&ai_path);
        UHPEngine { process }
    }
}

#[async_trait]
impl UHPCompliant for UHPEngine {
    async fn handle_command(&mut self, input: &str) -> String {
        self.process.send(input, true).await
    }
}
