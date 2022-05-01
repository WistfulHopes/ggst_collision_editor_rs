use core::panic;
use std::{path::{PathBuf, Path}, env::temp_dir, fs::{File, self, create_dir_all}, io::{Write, Read, BufReader}};
use arcsys::ggst::{pac::{GGSTPac, GGSTPacEntry}, jonbin::{GGSTJonBin, HitBox}};
use eframe::{egui::{self, Response, ComboBox, Sense, Frame}, emath::{Rect, Pos2}, epaint::{Color32, Stroke}};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use anyhow::Result as AResult;

use crate::open::open_file;

struct Box {
    x: String,
    y: String,
    w: String,
    h: String,
}


#[derive(Serialize, Deserialize)]
enum MetaKind {
    Pac(GGSTPac),
}

impl Default for Box {
    fn default() -> Self {
        Self {
            x: "0.0".to_owned(),
            y: "0.0".to_owned(),
            w: "0.0".to_owned(),
            h: "0.0".to_owned(),
        }
    }
}

pub struct BoxesWindow {
    path: PathBuf,
    pub jonbins: BTreeMap<String, GGSTJonBin>,
    selected: String,
    boxtype: String,
    offset_x: f32,
    offset_y: f32,
    last_cursor_pos: Pos2,
    current_box: Option<HitBox>,
    box_info: Box,
    box_index: u32,
}

impl Default for BoxesWindow {
    fn default() -> Self {
        Self {
            path: Default::default(),
            jonbins: Default::default(),
            selected: "".to_string(),
            boxtype: "".to_string(),
            offset_x: 640.0,
            offset_y: 802.0,
            last_cursor_pos: Default::default(),
            current_box: Default::default(),
            box_info: Default::default(),
            box_index: 0,
        }
    }
}

impl BoxesWindow {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        ComboBox::from_label("File list")
        .selected_text(format!("{:?}", self.selected))
        .width(150.0)
        .show_ui(ui, |ui| {
            for (name, _jonbin) in &self.jonbins {
                if ui.selectable_label(true, name)
                .clicked()
                {
                    if self.selected != ""
                    {
                        match self.write_jonb(){
                            Ok(_) => true,
                            Err(e) => panic!("Could not write jonbin! {}", e)
                        };
                    }
                    self.selected = name.to_string();
                };
            }
        });
        if self.selected != ""{
            self.box_list(ui);
            Frame::canvas(ui.style()).show(ui, |ui| {
                self.render_boxes(ui);
            });
        }
        else {
            ui.horizontal(|ui| {
                ui.label("Select a file from the file list!");
            });
        }
        ui.horizontal(|ui| {
            self.box_edit(ui, self.current_box);
        }).response
    }

    fn box_list(&mut self, ui: &mut egui::Ui) {
        let jonb = self.jonbins.get(&self.selected).unwrap();
        let mut index = 0;
        ui.horizontal(|ui| {
            ComboBox::from_label("Box list")
            .selected_text(format!("{} #{}", self.boxtype, self.box_index))
            .width(150.0)
            .show_ui(ui, |ui| {
                for hurtbox in &jonb.hurtboxes {
                    index += 1;
                    if ui.selectable_label(true, format!("Hurtbox #{}", index))
                    .clicked()
                    {
                        self.box_index = index;
                        self.boxtype = format!("Hurtbox");
                        self.box_info.x = format!("{}", hurtbox.rect.x_offset);
                        self.box_info.y = format!("{}", hurtbox.rect.y_offset);
                        self.box_info.w = format!("{}", hurtbox.rect.width);
                        self.box_info.h = format!("{}", hurtbox.rect.height);
                        self.current_box = Some(*hurtbox);
                    };
                }
                index = 0;
                for hitbox in &jonb.hitboxes {
                    index += 1;
                    if ui.selectable_label(true, format!("Hitbox #{}", index))
                    .clicked()
                    {
                        self.box_index = index;
                        self.boxtype = format!("Hitbox");
                        self.box_info.x = format!("{}", hitbox.rect.x_offset);
                        self.box_info.y = format!("{}", hitbox.rect.y_offset);
                        self.box_info.w = format!("{}", hitbox.rect.width);
                        self.box_info.h = format!("{}", hitbox.rect.height);
                        self.current_box = Some(*hitbox);
                    };
                }
            });
        });
    }

    fn box_edit(&mut self, ui: &mut egui::Ui, hitbox: Option<HitBox>) {
        match hitbox {
            None => {
                ui.vertical(|ui| {
                    ui.label("X Offset");
                    ui.text_edit_singleline(&mut "0.0");
                });
                ui.vertical(|ui| {
                    ui.label("Y Offset");
                    ui.text_edit_singleline(&mut "0.0");
                });
                ui.vertical(|ui| {
                    ui.label("Width");
                    ui.text_edit_singleline(&mut "0.0");
                });
                ui.vertical(|ui| {
                    ui.label("Height");
                    ui.text_edit_singleline(&mut "0.0");
                });
            },
            Some(mut hitbox) => {
                ui.vertical(|ui| {
                    ui.label("X Offset");
                    ui.text_edit_singleline(&mut self.box_info.x);
                });
                ui.vertical(|ui| {
                    ui.label("Y Offset");
                    ui.text_edit_singleline(&mut self.box_info.y);
                });
                ui.vertical(|ui| {
                    ui.label("Width");
                    ui.text_edit_singleline(&mut self.box_info.w);
                });
                ui.vertical(|ui| {
                    ui.label("Height");
                    ui.text_edit_singleline(&mut self.box_info.h);
                });
                hitbox.rect.x_offset = match self.box_info.x.parse::<f32>() {
                    Ok(float) => float,
                    Err(_) => hitbox.rect.x_offset,
                };
                hitbox.rect.y_offset = match self.box_info.y.parse::<f32>() {
                    Ok(float) => float,
                    Err(_) => hitbox.rect.y_offset,
                };
                hitbox.rect.width = match self.box_info.w.parse::<f32>() {
                    Ok(float) => float,
                    Err(_) => hitbox.rect.width,
                };
                hitbox.rect.height = match self.box_info.h.parse::<f32>() {
                    Ok(float) => float,
                    Err(_) => hitbox.rect.height,
                };
                self.current_box = Some(hitbox);
            }
        }
    }

    fn render_boxes(&mut self, ui: &mut egui::Ui) -> Response {
        let dark_mode = ui.visuals().dark_mode; 
        let faded_color = ui.visuals().window_fill();
        let faded_color = |color: Color32| -> Color32 {
            use egui::Rgba;
            let t = if dark_mode { 0.95 } else { 0.8 };
            egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
        };
        let jonb = self.jonbins.get_mut(&self.selected).unwrap();
        let (mut response, painter) = ui.allocate_painter(
            eframe::emath::Vec2 {
                x: (ui.available_width()),
                y: (ui.available_height() - 150.0)
            },
            Sense::click_and_drag()
        );

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if self.last_cursor_pos != Default::default()
            {
                let pointer_delta = pointer_pos - self.last_cursor_pos;
                self.offset_x += pointer_delta.x;
                self.offset_y += pointer_delta.y;
                response.mark_changed();
            }
            self.last_cursor_pos = pointer_pos;
        }
        else {
            self.last_cursor_pos = Default::default();
        }
        if response.clicked_by(egui::PointerButton::Secondary)
        {
            self.offset_x = 640.0;
            self.offset_y = 802.0;
        }

        let mut index = 0;

        for mut hurtbox in &mut jonb.hurtboxes {
            index += 1;
            if self.box_index == index && self.boxtype == format!("Hurtbox")
            {
                hurtbox.rect.x_offset = self.current_box.unwrap().rect.x_offset;
                hurtbox.rect.y_offset = self.current_box.unwrap().rect.y_offset;
                hurtbox.rect.width = self.current_box.unwrap().rect.width;
                hurtbox.rect.height = self.current_box.unwrap().rect.height;
            }
            painter.rect(
                Rect { min: Pos2{x: (hurtbox.rect.x_offset + self.offset_x ), 
                    y: (hurtbox.rect.y_offset + self.offset_y)}, 
                    max: Pos2{x: (hurtbox.rect.x_offset + hurtbox.rect.width + self.offset_x ), 
                    y: (hurtbox.rect.y_offset + hurtbox.rect.height + self.offset_y)} },
                0.0, 
                faded_color(Color32::DARK_GREEN),
                Stroke{width: 3.0, color: Color32::GREEN},
            );
        }

        index = 0;
        
        for mut hitbox in &mut jonb.hitboxes {
            index += 1;
            if self.box_index == index && self.boxtype == format!("Hitbox")
            {
                hitbox.rect.x_offset = self.current_box.unwrap().rect.x_offset;
                hitbox.rect.y_offset = self.current_box.unwrap().rect.y_offset;
                hitbox.rect.width = self.current_box.unwrap().rect.width;
                hitbox.rect.height = self.current_box.unwrap().rect.height;
            }
            painter.rect(
                Rect { min: Pos2{x: (hitbox.rect.x_offset + self.offset_x ), 
                    y: (hitbox.rect.y_offset + self.offset_y)}, 
                    max: Pos2{x: (hitbox.rect.x_offset + hitbox.rect.width + self.offset_x ), 
                    y: (hitbox.rect.y_offset + hitbox.rect.height + self.offset_y)} },
                0.0, 
                faded_color(Color32::RED),
                Stroke{width: 3.0, color: Color32::DARK_RED},
            );
        }
        response
    }

    fn reset(&mut self)
    {
        self.path = Default::default();
        self.jonbins = Default::default();
        self.selected = "".to_string();
        self.boxtype = "".to_string();
        self.offset_x = 640.0;
        self.offset_y = 802.0;
        self.last_cursor_pos = Default::default();
        self.current_box = Default::default();
        self.box_info = Default::default();
        self.box_index = 0;
    }

    pub fn add_hurtbox(&mut self)
    {
        let jonb = self.jonbins.get_mut(&self.selected).unwrap();
        let hurtbox = HitBox {kind: 0, rect: arcsys::ggst::jonbin::Rect {x_offset: 0.0, y_offset: 0.0, width: 0.0, height: 0.0}};
        jonb.hurtboxes.push(hurtbox);
    }

    pub fn add_hitbox(&mut self)
    {
        let jonb = self.jonbins.get_mut(&self.selected).unwrap();
        let hitbox = HitBox {kind: 1, rect: arcsys::ggst::jonbin::Rect {x_offset: 0.0, y_offset: 0.0, width: 0.0, height: 0.0}};
        jonb.hitboxes.push(hitbox);
    }

    pub fn remove_hurtbox(&mut self)
    {
        let jonb = self.jonbins.get_mut(&self.selected).unwrap();
        jonb.hurtboxes.pop();
    }

    pub fn remove_hitbox(&mut self)
    {
        let jonb = self.jonbins.get_mut(&self.selected).unwrap();
        jonb.hitboxes.pop();
    }

    pub fn open_file(&mut self, path: &PathBuf) -> bool {
        let pac = open_file(&path);
        if let Result::Ok(pac) = pac {
            self.reset();
            self.read_pac(path, pac);
            self.read_jonb();
            return true;
        }
        false
    }
    
    fn read_pac(&mut self, path: &PathBuf, pac: GGSTPac) {
        let mut dir = temp_dir();
        dir.push("GGSTCollisionEditorRS");

        let b: bool = Path::new(&dir).is_dir();
        if b {
            fs::remove_dir_all(&dir).unwrap();
        }
        let filename = path.file_stem().unwrap();
        dir.push(filename.to_str().unwrap());

        match create_dir_all(&dir){
            Ok(_) => {
                for i in &pac.files {
                    let mut content_file = match File::create(dir.join(&i.name)){
                        Ok(file) => file,
                        Err(_) => {
                            panic!("Could not create file at {:?}!", dir.join(&i.name));
                        },  
                    };
                    match content_file.write_all(&i.contents)
                    {
                        Ok(_) => {
                        },
                        Err(_) => {
                            panic!("Could not write file!");
                        },  
                    };
                }
                
                let meta_file = match File::create(dir.join("meta.json")) {
                    Ok(file) => file,
                    Err(_) => panic!("Could not create meta file!"),
                };
                let mut serializer = serde_json::Serializer::new(meta_file);        
                
                let meta = MetaKind::Pac(pac);
                meta.serialize(&mut serializer).expect("Failed to serialize meta file!");
                self.path = dir;
            },
            Err(e) => panic!("Could not create temp directory! {}", e),
        };
    }

    pub fn write_pac(&mut self, path: &PathBuf) -> AResult<()>
    {
        match self.write_jonb(){
            Ok(_) => true,
            Err(e) => panic!("Could not write jonbin! {}", e)
        };

        let mut meta_reader = BufReader::new(File::open(self.path.join("meta.json"))?);
        let meta: MetaKind = serde_json::from_reader(&mut meta_reader)?;
        
        match meta {
            MetaKind::Pac(mut pac) => {
                pac.files = pac
                .files
                .into_iter()
                .filter_map(|mut entry| {
                    let mut contents = Vec::new();
                    if File::open(self.path.join(&entry.name))
                        .and_then(|mut f| f.read_to_end(&mut contents))
                        .is_ok()
                    {
                        entry.contents = contents;
                        Some(entry)
                    } else {
                        println!("Failed to read {}! Excluding from PAC file", entry.name);
                        None
                    }
                })
                .collect::<Vec<GGSTPacEntry>>();

                let compressed = pac.to_bytes();
                self.write_repacked_file(path, compressed, "pac")?;
            }
        }

        Ok(())
    }

    fn write_repacked_file(
        &mut self,
        path: &PathBuf,
        bytes: Vec<u8>,
        extension: &str,
    ) -> Result<(), anyhow::Error> {
        let write_path = path.with_extension(extension);
        if write_path.exists() {
            println!(
                "{} is being overwritten!",
                write_path.file_name().unwrap().to_string_lossy()
            )
        }
        File::create(write_path)?.write_all(&bytes)?;
        Ok(())
    }

    fn read_jonb(&mut self)
    {
        let paths = match std::fs::read_dir(&self.path)
        {
            Ok(paths) => paths,
            Err(_) => return,
        };
        
        for path in paths {
            let file = match path{
                Ok(path) => path.path(),
                Err(_) => continue,
            };
            if file.is_file(){ 
                let mut file_buf = Vec::new();
                if let Err(e) = File::open(&file).and_then(|mut f| f.read_to_end(&mut file_buf)) {
                    println!("Error reading file {}: {}", file.display(), e);
                    return;
                };
                let byte_buf = file_buf.as_slice();
                match GGSTJonBin::parse(byte_buf){
                    Ok(jonb) => {
                        let filename = file.file_stem().unwrap();
                        self.jonbins.insert(filename.to_str().unwrap().to_string(),
                    jonb);
                    },
                    Err(_) => continue,
                };
            }
        }
    }

    fn write_jonb(&self) -> AResult<()>{
        let write_path = self.path.join(&self.selected);
        let bytes = GGSTJonBin::to_bytes(self.jonbins.get(&self.selected).unwrap());
        if write_path.exists() {
            println!(
                "{} is being overwritten!",
                write_path.file_name().unwrap().to_string_lossy()
            )
        }
        File::create(write_path)?.write_all(&bytes)?;
        Ok(())
    }
}