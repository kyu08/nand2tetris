use std::path::{Path, PathBuf};
use translator::VMProgram;

mod translator;

fn main() {
    let command_line_args: Vec<String> = std::env::args().collect();
    if command_line_args.len() != 2 {
        println!("Usage: vm_translator <filepath>");
        return;
    }
    let source_file_path = std::path::PathBuf::from(&command_line_args[1]);
    let (target_files, output_file_path) = get_target_files(&source_file_path).unwrap();

    let vm_files = target_files
        .into_iter()
        .map(|target| {
            let file_name_without_ext = target.file_stem().unwrap().to_string_lossy().to_string();
            parse(target, file_name_without_ext)
        })
        .collect();

    let combined_assembly = VMProgram::combine_and_assemble(vm_files);
    let _ = std::fs::write(output_file_path, combined_assembly);
}

// 任意のpathを渡せるようにしておくとUTが書きやすいので切り出しておく
fn parse(path: PathBuf, file_name: String) -> translator::VMProgram {
    let content = std::fs::read_to_string(path.clone()).expect("Failed to read file");
    translator::VMProgram::new(file_name, content)
}

fn get_target_files(input_path: &Path) -> Option<(Vec<PathBuf>, PathBuf)> {
    // inputがファイルだったら.vmかどうか判定して(target_files, output_file_path)を返す
    if input_path.is_file() {
        if input_path.extension().unwrap() != "vm" {
            return None;
        }
        return Some((vec![input_path.to_path_buf()], input_path.with_extension("asm")));
    }
    if !input_path.is_dir() {
        return None;
    }

    // inputがフォルダだったら.vmファイルを探してきて(target_files, output_file_path)を返す
    let mut result = vec![];
    for e in std::fs::read_dir(input_path).unwrap() {
        let e_path = e.unwrap().path();
        if e_path.extension().unwrap() == "vm" {
            result.push(e_path);
        }
    }

    Some((result, input_path.join(input_path.file_name().unwrap()).with_extension("asm")))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn test_get_target_files() {
        // cleanup before test
        let test_root_dir = std::env::temp_dir().join("nand2tetris08");
        // error will be returned if the directory does not exist.
        let _ = std::fs::remove_dir_all(&test_root_dir);
        std::fs::create_dir(&test_root_dir).unwrap();

        // fileのパスを渡す(拡張子がvm以外)
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let file_path = test_target_dir.join("foo.md");
            std::fs::File::create(&file_path).unwrap();
            assert_eq!(get_target_files(&file_path), None);
        }

        // fileのパスを渡す(拡張子がvm)
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let file_path = test_target_dir.join("foo.vm");
            std::fs::File::create(&file_path).unwrap();
            assert_eq!(get_target_files(&file_path), Some((vec![file_path], test_target_dir.join("foo.asm"))));
        }

        // dirのパスを渡す
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let parent_dir = test_target_dir.join("Foo");
            std::fs::create_dir(&parent_dir).unwrap();
            let main = parent_dir.join("Main.vm");
            std::fs::File::create(&main).unwrap();
            let child1 = parent_dir.join("Child1.vm");
            std::fs::File::create(&child1).unwrap();
            let child2 = parent_dir.join("Child2.vm");
            std::fs::File::create(&child2).unwrap();
            let other = parent_dir.join("README.md");
            std::fs::File::create(&other).unwrap();

            let result = get_target_files(&parent_dir.clone()).unwrap();
            assert_eq!(result.0.len(), 3);
            assert_eq!(result.0.contains(&main), true);
            assert_eq!(result.0.contains(&child1), true);
            assert_eq!(result.0.contains(&child2), true);
            assert_eq!(result.0.contains(&other), false);
            assert_eq!(result.1, parent_dir.join("Foo.asm"));
        }

        let _ = std::fs::remove_dir_all(&test_root_dir);
    }
}
