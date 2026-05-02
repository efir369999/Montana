use std::path::PathBuf;
use std::process::ExitCode;

use montana_node::commands::{init, inspect, start, status, time};
use montana_node::NodeError;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 2 {
        eprintln!("{}", help_text());
        return ExitCode::from(2);
    }

    let result = match argv[1].as_str() {
        "init" => parse_init(&argv[2..]).and_then(init::run),
        "inspect" => parse_inspect(&argv[2..]).and_then(inspect::run),
        "status" => parse_status(&argv[2..]).and_then(status::run),
        "time" => parse_time(&argv[2..]).and_then(time::run),
        "start" => parse_start(&argv[2..]).and_then(start::run),
        "help" | "-h" | "--help" => {
            println!("{}", help_text());
            return ExitCode::SUCCESS;
        },
        other => {
            eprintln!("неизвестная команда: {other}");
            eprintln!();
            eprintln!("{}", help_text());
            return ExitCode::from(2);
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ошибка: {e}");
            ExitCode::from(1)
        },
    }
}

fn help_text() -> String {
    String::from(
        "montana-node — узел Montana (singleton mode, без сетевого слоя)\n\
         \n\
         Использование:\n\
         \n\
           montana-node init    [--data-dir <PATH>] [--mnemonic \"<24 слова>\"]\n\
                                 [--entropy <hex32>] [--force]\n\
         \n\
           montana-node inspect [--data-dir <PATH>] [--reveal-master-seed]\n\
         \n\
           montana-node status  [--data-dir <PATH>]\n\
         \n\
           montana-node time    [--data-dir <PATH>]\n\
         \n\
           montana-node start   [--data-dir <PATH>] [--max-windows <N>]\n\
                                 [--d-test-override <N>]\n\
         \n\
         Команды:\n\
         \n\
           init      Создать identity (мнемоника + ключи) и сохранить в identity.bin.\n\
                     Если ни --mnemonic, ни --entropy не указаны — генерируется\n\
                     случайная энтропия через системный ГСЧ.\n\
         \n\
           inspect   Прочитать identity.bin и вывести account_id / node_id /\n\
                     fingerprint. Секретные ключи на экран не выводятся.\n\
         \n\
           status    Показать содержимое локального state: AccountTable,\n\
                     NodeTable, CandidatePool, статус узла, current_window,\n\
                     phase lifecycle (Bootstrap/CandidateVdf/Registered/Active),\n\
                     supply, балансы.\n\
         \n\
           time      Показать current_window локального узла, ближайшее\n\
                     selection-окно, эпоху τ₂.\n\
         \n\
           start     БОЕВОЙ РЕЖИМ — запуск узла Montana через canonical\n\
                     apply_proposal pipeline. Узел проходит lifecycle:\n\
                       Bootstrap → CandidateVdf (тикает VDF до vdf_chain_length\n\
                                                   ≥ τ₂ = 20160 окон, ~10 часов\n\
                                                   wall-clock на M-class Mac)\n\
                       CandidateVdf → Registered (формирует NodeRegistration\n\
                                                   через apply_noderegistrations_batch)\n\
                       Registered → Active (через apply_selection_event на\n\
                                            следующем W % 336 == 0)\n\
                       Active: per окно VdfReveal + BundledConfirmation +\n\
                               ProposalHeader + apply_proposal + archive_proposal,\n\
                               state_root self-verify, эмиссия 13 Ɉ оператору\n\
                     Ctrl-C — корректная остановка с сохранением state.\n\
         \n\
         Опции:\n\
         \n\
           --data-dir <PATH>          Каталог данных узла. По умолчанию на macOS:\n\
                                      $HOME/Library/Application Support/Montana/node\n\
           --mnemonic \"...\"           Восстановить identity из 24-словной фразы.\n\
           --entropy <hex32>          Использовать 32-байтную энтропию (64 hex).\n\
           --force                    Перезаписать существующий identity.bin.\n\
           --reveal-master-seed       В inspect: показать полный master_seed.\n\
           --max-windows <N>          В start: остановиться после N окон.\n\
           --d-test-override <N>      В start: TEST-ONLY override D = N итераций.\n\
                                      Production использует params.d0 = 300_000_000.\n\
                                      Override используется в тестах для скорости.\n",
    )
}

fn parse_init(args: &[String]) -> Result<init::InitArgs, NodeError> {
    let mut data_dir: Option<PathBuf> = None;
    let mut mnemonic: Option<String> = None;
    let mut entropy_hex: Option<String> = None;
    let mut force = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(expect_value(args, i, "--data-dir")?));
                i += 2;
            },
            "--mnemonic" => {
                mnemonic = Some(expect_value(args, i, "--mnemonic")?.to_string());
                i += 2;
            },
            "--entropy" => {
                entropy_hex = Some(expect_value(args, i, "--entropy")?.to_string());
                i += 2;
            },
            "--force" => {
                force = true;
                i += 1;
            },
            other => {
                return Err(NodeError::InvalidArguments(format!(
                    "неизвестный флаг для init: {other}"
                )))
            },
        }
    }
    Ok(init::InitArgs {
        data_dir,
        mnemonic,
        entropy_hex,
        force,
    })
}

fn parse_inspect(args: &[String]) -> Result<inspect::InspectArgs, NodeError> {
    let mut data_dir: Option<PathBuf> = None;
    let mut reveal_master_seed = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(expect_value(args, i, "--data-dir")?));
                i += 2;
            },
            "--reveal-master-seed" => {
                reveal_master_seed = true;
                i += 1;
            },
            other => {
                return Err(NodeError::InvalidArguments(format!(
                    "неизвестный флаг для inspect: {other}"
                )))
            },
        }
    }
    Ok(inspect::InspectArgs {
        data_dir,
        reveal_master_seed,
    })
}

fn parse_status(args: &[String]) -> Result<status::StatusArgs, NodeError> {
    let mut data_dir: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(expect_value(args, i, "--data-dir")?));
                i += 2;
            },
            other => {
                return Err(NodeError::InvalidArguments(format!(
                    "неизвестный флаг для status: {other}"
                )))
            },
        }
    }
    Ok(status::StatusArgs { data_dir })
}

fn parse_time(args: &[String]) -> Result<time::TimeArgs, NodeError> {
    let mut data_dir: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(expect_value(args, i, "--data-dir")?));
                i += 2;
            },
            other => {
                return Err(NodeError::InvalidArguments(format!(
                    "неизвестный флаг для time: {other}"
                )))
            },
        }
    }
    Ok(time::TimeArgs { data_dir })
}

fn parse_start(args: &[String]) -> Result<start::StartArgs, NodeError> {
    let mut data_dir: Option<PathBuf> = None;
    let mut max_windows: Option<u64> = None;
    let mut d_test_override: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(expect_value(args, i, "--data-dir")?));
                i += 2;
            },
            "--max-windows" => {
                max_windows = Some(expect_value(args, i, "--max-windows")?.parse().map_err(
                    |_| NodeError::InvalidArguments("--max-windows должен быть u64".into()),
                )?);
                i += 2;
            },
            "--d-test-override" => {
                d_test_override = Some(
                    expect_value(args, i, "--d-test-override")?
                        .parse()
                        .map_err(|_| {
                            NodeError::InvalidArguments("--d-test-override должен быть u64".into())
                        })?,
                );
                i += 2;
            },
            other => {
                return Err(NodeError::InvalidArguments(format!(
                    "неизвестный флаг для start: {other}"
                )))
            },
        }
    }
    Ok(start::StartArgs {
        data_dir,
        max_windows,
        d_test_override,
    })
}

fn expect_value<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str, NodeError> {
    args.get(i + 1)
        .map(|s| s.as_str())
        .ok_or_else(|| NodeError::InvalidArguments(format!("флаг {flag} требует значения")))
}
