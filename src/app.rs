use serde::{Deserialize, Serialize};
use tui::widgets::TableState;

#[derive(Serialize, Deserialize)]
pub struct System<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub status: &'a str,
    pub ports: Vec<&'a str>,
}

#[derive(Serialize, Deserialize)]
pub struct Data<'a> {
    pub title: &'a str,
    pub systems: Vec<System<'a>>,
}

pub struct App<'a> {
    pub state: TableState,
    pub title: &'a str,
    pub systems: Vec<System<'a>>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, systems: Vec<System<'a>>) -> App<'a> {
        App {
            state: TableState::default(),
            title,
            systems: systems,
        }
    }
    pub fn default() -> App<'a> {
        App {
            state: TableState::default(),
            title: "DEFAULT",
            systems: vec![System {
                name: "",
                address: "",
                status: "",
                ports: vec![],
            }],
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.systems.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.systems.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
