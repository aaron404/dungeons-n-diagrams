use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use windows_sys::Win32::Foundation::DNS_ERROR_UNKNOWN_RECORD_TYPE;

const DATA_DIR_ROOT: &str =
    r"G:\Program Files (x86)\Steam\steamapps\common\Last Call BBS\Content\Packed\textures\tokyo";

const MONSTERS_DIR: &str = r"G:\Program Files (x86)\Steam\steamapps\common\Last Call BBS\Content\Packed\textures\tokyo\monsters";

const DATA_FILE_NAMES: [&str; 2] = ["idle", "dance"];
const TEX_SUFFIX: &str = ".array.tex";

pub fn list_monsters() {
    // print header
    println!("{: <16}", "texture name");
    for tex_name in DATA_FILE_NAMES.iter() {
        for entry in std::fs::read_dir(MONSTERS_DIR)
            .expect("Failed to find data dir")
            .filter_map(|d| d.ok())
        {
            let monster_name = entry.file_name();
            let monster_name = monster_name.to_str().unwrap();

            if monster_name != "chest" {
                // continue;
            }
            let mut path = PathBuf::from(MONSTERS_DIR);
            path.push(entry.file_name());

            let fname = PathBuf::from(format!("{}{TEX_SUFFIX}", tex_name));
            let mut full_path = path.clone();
            full_path.push(&fname);

            let sprite_name = format!("{}-{tex_name}", monster_name);
            println!("{: <16}", sprite_name);
            parse_texture(&full_path, &sprite_name);
        }
        if *tex_name == "idle" {
            println!("------------------------------------------------------");
        }
    }
}

fn parse_texture(path: &PathBuf, sprite_name: &String) -> io::Result<()> {
    // let mut rdr = Cursor::new(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    // assert_eq!(16909060, rdr.read_u32::<BigEndian>().unwrap());
    // assert_eq!(84281096, rdr.read_u32::<BigEndian>().unwrap());

    let mut buffer = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buffer).unwrap();

    // println!("{:?}", buffer.chunks(12).next().unwrap());

    let mut rdr = Cursor::new(buffer);
    // rdr.read_u16::<LittleEndian>().unwrap();

    // let mut frames = vec![];
    let mut compressed = Vec::new();

    let magic = rdr.read_u32::<LittleEndian>().unwrap();
    rdr.seek(SeekFrom::Current(4)).unwrap();

    let mut i = 0;

    // let mut frames = vec![];
    let mut sizes = HashSet::new();

    while let Ok(width) = rdr.read_u32::<LittleEndian>() {
        let height = rdr.read_u32::<LittleEndian>().unwrap();
        if true {
            rdr.seek(SeekFrom::Current(56)).unwrap();
        } else {
            println!(
                "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:.1} {:>4} {:.1} {:.1} {:>4}",
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_f32::<LittleEndian>().unwrap(),
                rdr.read_u32::<LittleEndian>().unwrap(),
            );
        }

        let payload_size = rdr.read_u32::<LittleEndian>().unwrap();
        compressed.resize(payload_size as usize, 0);
        rdr.read_exact(&mut compressed).unwrap();

        let texture = lz4_flex::decompress(&compressed, 1000000).unwrap();
        let img = image::RgbaImage::from_raw(width, height, texture).unwrap();
        image::imageops::flip_vertical(&img);
        // image::imageops::resize(&img, 32, 32, FilterType)
        // frames.push(img);

        sizes.insert((width, height));

        // img.save(format!("monsters/{sprite_name}-{i:02}.png"))
        //     .unwrap();
        i += 1;
        // println!("  {magic} {width}x{height}: {payload_size}");
    }

    println!("{sizes:?}");
    Ok(())
}
