use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

mod graph;
mod edge;
mod path;
mod pathfinder;

fn main() {
    println!("Hello, world!");
    let tile_map = TileMap::new(20);
    tile_map.print_grid();
}

#[derive(Debug)]
pub struct TileMap {
    size: u32,
    map: Vec<Tile>,
    lookup: HashMap<Vector2, usize>,
}

impl TileMap {
    pub fn new(size: u32) -> Self {
        let mut map = Vec::new();
        let mut lookup = HashMap::<Vector2, usize>::new();
        for i in 0..size {
            for w in 0..size {
                let pos = Vector2 { a: i, b: w };
                let idx = map.len();
                lookup.insert(pos.clone(), idx);
                let mut connections = Vec::new();
                let minus_x: u32 = i.saturating_sub(1);
                let plus_x: u32 = (i + 1).min(size - 1);
                let minus_y: u32 = w.saturating_sub(1);
                let plus_y: u32 = (w + 1).min(size - 1);

                let horizontal: Vec<u32> = vec![minus_x, plus_x];
                let vertical: Vec<u32>= vec![minus_y, plus_y];
                for x in &horizontal {
                    for y in &vertical {
                        let _pos = Vector2 {
                            a: x.clone(),
                            b: y.clone(),
                        };
                        connections.push(_pos.clone());
                    }
                }

                let tile = Tile {
                    tile_type: if random_bool() {
                        TileType::Path
                    } else {
                        TileType::Block
                    },
                    connections: connections,
                    position: pos,
                };
                map.push(tile);
            }
        }

        TileMap { size, map, lookup }
    }

    pub fn print_grid(&self) {
        for y in 0..self.size {
            for x in 0..self.size {
                let index = (y * self.size + x) as usize;
                let tile = &self.map[index];
                let ch = match tile.tile_type {
                    TileType::Path => "  ", // Two spaces for open path
                    TileType::Block => "██", // Block characters for blocked tile
                };
                print!("{}", ch);
            }
            println!();
        }
    }

}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Vector2 {
    a: u32,
    b: u32,
}

#[derive(Debug)]
pub enum TileType {
    Path,
    Block,
}

#[derive(Debug)]
pub struct Tile {
    pub tile_type: TileType,
    pub connections: Vec<Vector2>,
    pub position: Vector2,
}

fn random_bool() -> bool {
    // Obtém o tempo atual em nanossegundos desde a época UNIX.
    let now = SystemTime::now();
    let duration = now
        .duration_since(UNIX_EPOCH)
        .expect("Falha ao obter a duração desde a época UNIX");
    let nanos = duration.subsec_nanos();

    // Retorna 'true' ou 'false' com base na paridade dos nanossegundos.
    nanos % 2 == 0
}
