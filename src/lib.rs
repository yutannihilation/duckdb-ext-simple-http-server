extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::CString,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock, Mutex, OnceLock,
    },
};

use tokio::runtime::Runtime;
use warp::Filter;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

// cf. https://github.com/duckdb/duckdb-ui/blob/25ff9b6fa0f37853b92e9f5a7ea517bf78656f3f/src/ui_extension.cpp#L14-L20
#[cfg(target_os = "windows")]
fn open_browser(url: &str) {
    // It seems new("start").arg(url) doesn't work...

    Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()
        .unwrap();
}
#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    Command::new("open").arg(url).spawn().unwrap();
}
#[cfg(target_os = "linux")]
fn open_browser(url: &str) {
    Command::new("xdg-open").arg(url).spawn().unwrap();
}

#[repr(C)]
struct HelloBindData {}

#[repr(C)]
struct HelloInitData {
    done: AtomicBool,
}

struct HelloVTab;

impl VTab for HelloVTab {
    type InitData = HelloInitData;
    type BindData = HelloBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        bind.add_result_column("result", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        Ok(HelloBindData {})
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        let get = warp::get().map(|| {
            // let conn = Connection::open_in_memory().unwrap();
            let mut tables = vec![];
            CONN.get()
                .unwrap()
                .lock()
                .unwrap()
                .pragma_query(None, "show_tables", |row| {
                    let t: String = row.get(0)?;
                    tables.push(format!("<li>{t}</li>"));
                    Ok(())
                })
                .unwrap();

            let tables = tables.join("\n");

            warp::reply::html(format!(
                r#"
<html>
  <head>
    <title>あああ</title>
  </head>
  <body>
    <div>
<ul>
{tables}
</ul>
    </div>
  </body>
</html>
"#
            ))
        });

        RUNTIME.spawn(async move { warp::serve(get).run(([127, 0, 0, 1], 3030)).await });

        open_browser("http://127.0.0.1:3030");

        Ok(HelloInitData {
            done: AtomicBool::new(false),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
        } else {
            let vector = output.flat_vector(0);
            let url = "http://127.0.0.1:3030"; // TODO
            let result = CString::new(format!("URL => {}", url))?;
            vector.insert(0, result);
            output.set_len(1);
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![])
    }
}

const EXTENSION_NAME: &str = env!("CARGO_PKG_NAME");

static CONN: OnceLock<Mutex<Connection>> = OnceLock::new();

#[duckdb_entrypoint_c_api()]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<HelloVTab>(EXTENSION_NAME)
        .expect("Failed to register hello table function");

    // TODO: handle error
    let _ = CONN.set(Mutex::new(con));

    Ok(())
}
