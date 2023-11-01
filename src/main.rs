mod clean;
mod compile;
mod config;
mod init;
mod log;
mod opts;
mod templates;
mod utils;

use opts::{Command, Opts};

fn main() {
    let opts = Opts::create();
    run(opts);
}

fn run(opts: Opts) {
    match opts.args.command {
        Command::Init { template } => init::init(opts.cwd, opts.config, template),
        Command::Compile {
            main_file: cli_main_file,
        } => {
            let main_file = cli_main_file.unwrap_or(opts.config.main_file.clone());
            compile::compile(opts.cwd, opts.config, &main_file)
        }
        Command::Clean {
            main_file: cli_main_file,
        } => {
            let main_file = cli_main_file.unwrap_or(opts.config.main_file.clone());
            clean::clean(opts.cwd, opts.config, &main_file)
        }
        Command::Log { log_file } => {
            log::print_log(opts.cwd, &log_file.unwrap_or(opts.config.main_file))
        }
        Command::Templates { template_command } => match template_command {
            opts::TemplateCommand::Add {
                paths,
                symlink,
                force,
                rename,
            } => templates::add_paths(opts.cwd, opts.config, paths, symlink, force, rename),
            opts::TemplateCommand::AddRepo { url, path, force } => {
                templates::add_repo(opts.cwd, opts.config, url, path, force)
            }
            opts::TemplateCommand::List => templates::list_templates(opts.config),
        },
        Command::Config {
            config_command,
            global,
        } => match config_command {
            opts::ConfigCommand::Create { force } => config::create(&opts.cwd, global, force),
            opts::ConfigCommand::Show => config::show(opts.config, global),
        },
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::{
        opts::{Config, Opts},
        run,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    const TEST_DIR: &str = "./__tmp_test_dir__/";

    const DATA_DIR: &str = "data_dir/";
    const TEMPLATES_DIR: &str = "templates_dir/";
    const TEMP_DIR: &str = "temp_dir/";
    const CWD_DIR: &str = "cwd/";

    struct TestContext;

    impl TestContext {
        fn new(config: &Config) -> Self {
            if PathBuf::from(TEST_DIR).exists() {
                fs::remove_dir_all(TEST_DIR).unwrap();
            }

            fs::create_dir(TEST_DIR).unwrap();
            fs::create_dir(TEST_DIR.to_string() + CWD_DIR).unwrap();
            fs::create_dir(&config.data_dir).unwrap();
            fs::create_dir(&config.templates_dir).unwrap();
            fs::create_dir(&config.temp_dir).unwrap();
            Self
        }
    }
    impl Drop for TestContext {
        fn drop(&mut self) {
            fs::remove_dir_all(TEST_DIR).unwrap();
        }
    }
    fn setup(args: Vec<&str>) -> (TestContext, Opts) {
        let cwd = PathBuf::from(TEST_DIR).join(CWD_DIR);

        // main_file: todo!(),
        // compile_cmd: todo!(),
        // clean_cmd: todo!(),
        // data_dir: todo!(),
        // templates_dir: todo!(),
        // config_file: todo!(),
        // temp_dir: todo!(),
        let mut config = Config::default();
        config.data_dir = PathBuf::from(TEST_DIR.to_string() + DATA_DIR);
        config.templates_dir = PathBuf::from(TEST_DIR.to_string() + TEMPLATES_DIR);
        config.temp_dir = PathBuf::from(TEST_DIR.to_string() + TEMP_DIR);

        (
            TestContext::new(&config),
            Opts::create_mock(args, config, cwd),
        )
    }

    macro_rules! setup {
        ($($x:expr),+) => {
            setup(vec![$($x),+])
        }
    }

    #[test]
    #[serial]
    fn test_compile_and_clean() {
        #[allow(unused_variables)]
        let (ctx, opts) = setup!("compile");
        fs::copy("./tests/main1.tex", opts.cwd.join("main.tex")).unwrap();
        run(opts.clone());
        assert!(opts.cwd.join("main.log").exists());
        assert!(opts.cwd.join("main.out").exists());
        let clean_opts = Opts::create_mock(vec!["clean"], opts.config, opts.cwd);
        run(clean_opts.clone());
        assert!(!clean_opts.cwd.join("main.log").exists());
        assert!(!clean_opts.cwd.join("main.out").exists());
    }

    #[test]
    #[serial]
    fn test_add_and_compile() {
        #[allow(unused_variables)]
        let (ctx, opts) = setup!("templates", "add", "templates");
        let add_opts = opts.clone();
        let init_opts = Opts::create_mock(
            vec!["init", "-t", "templates/minimal"],
            add_opts.config.clone(),
            add_opts.cwd.clone(),
        );
        let compile_opts = Opts::create_mock(
            vec!["compile"],
            add_opts.config.clone(),
            add_opts.cwd.clone(),
        );
        run(add_opts);
        run(init_opts);
        run(compile_opts);
        assert!(opts.cwd.join("main.pdf").exists())
    }
}
