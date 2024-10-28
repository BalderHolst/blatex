use serial_test::serial;

use crate::{
    opts::{Config, Opts, RemoteTemplate},
    run, utils,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const TEST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__tmp_test_dir__/");

const DATA_DIR: &str = "data_dir/";
const CONFIG_DIR: &str = "config_dir/";
const TEMPLATES_DIR: &str = "templates_dir/";
const TEMP_DIR: &str = "temp_dir/";
const CWD_DIR: &str = "cwd/";

struct TestContext;

impl TestContext {
    fn new(config: &Config) -> Self {
        if PathBuf::from(TEST_DIR).exists() {
            utils::remove_dir_all(Path::new(TEST_DIR));
        }

        utils::create_dir(Path::new(TEST_DIR));
        utils::create_dir(Path::new(&(TEST_DIR.to_string() + CWD_DIR)));
        utils::create_dir(&config.data_dir);
        utils::create_dir(&config.templates_dir);
        utils::create_dir(&config.temp_dir);

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

    let mut config = Config::default();
    config.root = cwd.clone();
    config.data_dir = PathBuf::from(TEST_DIR.to_string() + DATA_DIR);
    config.config_file = PathBuf::from(TEST_DIR.to_string() + CONFIG_DIR + "config.toml");
    config.templates_dir = PathBuf::from(TEST_DIR.to_string() + TEMPLATES_DIR);
    config.temp_dir = PathBuf::from(TEST_DIR.to_string() + TEMP_DIR);

    // Silence latex compilation
    config.compile_cmd = config.compile_cmd + " > /dev/null";
    config.clean_cmd = config.clean_cmd + " > /dev/null";

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
    println!("test_compile_and_clean");
    #[allow(unused_variables)]
    let (ctx, opts) = setup!("compile");
    fs::copy("./tests/main1.tex", opts.cwd.join("main.tex")).unwrap();
    run(opts.clone());
    assert!(opts.cwd.join("main.log").exists());
    assert!(opts.cwd.join("main.pdf").exists());
    let clean_opts = Opts::create_mock(vec!["clean"], opts.config, opts.cwd);
    run(clean_opts.clone());
    assert!(!clean_opts.cwd.join("main.log").exists());
    assert!(!clean_opts.cwd.join("main.out").exists());
    assert!(clean_opts.cwd.join("main.pdf").exists());
}

#[test]
#[serial]
fn test_compile_from_subfolder() {
    println!("test_compile_and_clean");
    #[allow(unused_variables)]
    let (ctx, init_opts) = setup!("init");
    fs::copy("./tests/main1.tex", init_opts.cwd.join("main.tex")).unwrap();

    // Create `.blatex.toml`
    run(init_opts.clone());

    let mut compile_opts = Opts::create_mock(vec!["compile"], init_opts.config, init_opts.cwd);

    // Create sub directory
    fs::create_dir(compile_opts.cwd.join("subdir")).unwrap();

    // Go into sub directory
    compile_opts.cwd = compile_opts.cwd.join("subdir");

    run(compile_opts.clone());
    assert!(!compile_opts.cwd.join("main.log").exists());
    assert!(!compile_opts.cwd.join("main.pdf").exists());
    assert!(compile_opts.config.root.join("main.log").exists());
    assert!(compile_opts.config.root.join("main.pdf").exists());
}

#[test]
#[serial]
fn test_add_and_compile() {
    println!("test_add_and_compile");
    let (_ctx, opts) = setup!("template", "add", "../../templates/minimal.zip");
    let add_opts = opts.clone();
    let init_opts = Opts::create_mock(
        vec!["init", "-t", "minimal"],
        add_opts.config.clone(),
        add_opts.cwd.clone(),
    );
    let compile_opts = Opts::create_mock(
        vec!["compile"],
        add_opts.config.clone(),
        add_opts.cwd.clone(),
    );
    println!("Adding...");
    run(add_opts);
    println!("Initializing...");
    run(init_opts);
    println!("Compiling...");
    run(compile_opts);
    assert!(opts.cwd.join("main.pdf").exists())
}

#[test]
#[serial]
#[ignore]
fn test_add_repo_branch() {
    println!("test_add_repo_zip_and_compile");
    #[allow(unused_variables)]
    let (ctx, opts) = setup!(
        "template",
        "add-repo",
        "https://github.com/cainmagi/Latex-Templates",
        "-b",
        "elegant-report"
    );
    let add_opts = opts.clone();
    let init_opts = Opts::create_mock(
        vec![
            "init",
            "-t",
            "elegant-report@Latex-Templates",
            "-m",
            "gReport.tex",
        ],
        add_opts.config.clone(),
        add_opts.cwd.clone(),
    );

    println!("Adding...");
    run(add_opts);

    println!("Initializing...");
    run(init_opts);

    // TODO: The mocked options do not take local config into a count. I can therefore not run
    // the `compile` command here, even though it does work in a real use case.

    assert!(opts.cwd.join("gReport.pdf").exists())
}

#[test]
#[serial]
#[ignore]
fn test_add_repo_zip_and_compile() {
    println!("test_add_repo_zip_and_compile");
    #[allow(unused_variables)]
    let (ctx, opts) = setup!(
        "template",
        "add-repo",
        "https://github.com/BalderHolst/blatex",
        "-p",
        "templates/basic.zip"
    );
    let add_opts = opts.clone();
    let init_opts = Opts::create_mock(
        vec!["init", "-t", "basic"],
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

#[test]
#[serial]
fn rename_add_templates() {
    println!("rename_add_templates");
    #[allow(unused_variables)]
    // Rename file
    let (ctx, opts) = setup!(
        "template",
        "add",
        "-r",
        "template_file",
        "../../templates/basic.zip"
    );

    run(opts.clone());

    assert!(opts
        .config
        .templates_dir
        .join("template_file.zip")
        .is_file());
    assert!(!opts.config.templates_dir.join("basic.zip").is_file());

    // Rename file to file within new directory
    let opts = Opts::create_mock(
        vec![
            "template",
            "add",
            "-r",
            "dir/within/dir/awesome_template",
            "../../templates/basic.zip",
        ],
        opts.config,
        opts.cwd,
    );
    run(opts.clone());
    assert!(opts
        .config
        .templates_dir
        .join("dir/within/dir/awesome_template.zip")
        .is_file());
}

#[test]
#[serial]
#[ignore]
fn test_remote_template_config() {
    println!("test_remote_template_config");

    let (_ctx, mut opts) = setup!("init", "-t", "test-template");

    let config_dir = opts.config.config_file.parent().unwrap();
    fs::create_dir_all(config_dir).unwrap();

    opts.config.remote_templates.insert(
        "test-template".to_string(),
        RemoteTemplate {
            url: "https://github.com/BalderHolst/blatex".to_string(),
            path: Some(PathBuf::from("tests")),
            branch: Some("main".to_string()),
            config: {
                let mut config = opts.config.clone();
                config.main_file = PathBuf::from("main1.tex");
                config
            },
        },
    );

    run(opts.clone());
    assert!(opts.config.root.join("main1.pdf").exists())
}
