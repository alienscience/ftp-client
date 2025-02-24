//! Tests for the FTP client crate, all the tests
//! are made with real sample FTP servers.
//!
//! Tests that start with external_ are run with
//! external FTP servers, the others are run
//! with a local dockerize server that you should start.
use ftp_client::error::Error as FtpError;
use ftp_client::prelude::*;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

#[test]
fn external_name_listing() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;

    assert_eq!(
        vec!["/pub".to_string(), "/readme.txt".to_string()],
        client.list_names("/")?
    );
    Ok(())
}

#[test]
fn external_pwd() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.cwd("/pub")?;
    let dir = client.pwd()?;
    assert!(dir.contains("/pub"));

    Ok(())
}

#[test]
fn external_site() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.site_parameters()?;

    Ok(())
}

#[test]
fn external_file_retrieval() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    let readme_file = client.retrieve_file("/readme.txt")?;
    // Taken previously and unlikely to change
    let file_size = 403;

    assert_eq!(readme_file.len(), file_size);
    Ok(())
}

#[test]
fn external_cwd() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.cwd("/pub/example")?;

    // The /pub/example dir has many files
    let names = client.list_names("")?;
    assert!(names.len() > 3);

    Ok(())
}

#[test]
fn external_cdup() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    let initial_names = client.list_names("")?;
    client.cwd("/pub/example")?;

    // Go up two times
    client.cdup()?;
    client.cdup()?;

    let final_names = client.list_names("")?;
    assert_eq!(initial_names, final_names);

    Ok(())
}

#[test]
fn external_logout() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.logout()
}

#[test]
fn external_noop() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.noop()
}

#[test]
fn external_help() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    client.help()
}

#[test]
fn external_store() -> Result<(), FtpError> {
    let mut client = Client::connect(
        "speedtest4.tele2.net",
        "anonymous",
        "anonymous@anonymous.com",
    )?;
    let file_data = b"Some data for you";
    let file_name = "/upload/readyou.txt";

    client.store(file_name, file_data)
}

#[test]
fn external_store_unique() -> Result<(), FtpError> {
    let mut client = Client::connect(
        "speedtest4.tele2.net",
        "anonymous",
        "anonymous@anonymous.com",
    )?;
    client.cwd("/upload/")?;
    let file_data = b"Some data for you";
    client.store_unique(file_data)?;

    Ok(())
}

#[test]
fn external_system() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    // Should be Windows_NT but we don't need to check that..
    // since we don't want to break tests if the server changes OS
    let _system_name = client.system()?;

    Ok(())
}

#[test]
#[ignore]
fn external_ipv6() -> Result<(), FtpError> {
    let mut client = Client::connect(
        "speedtest6.tele2.net",
        "anonymous",
        "anonymous@anonymous.com",
    )?;

    let data = b"DATA";
    let file_path = "/upload/readyou.txt";
    client.store(file_path, data)
}

#[test]
fn external_tls() -> Result<(), FtpError> {
    let mut client = Client::connect("test.rebex.net", "demo", "password")?;
    // Run random command just to assert we are communicating
    let _system_name = client.system()?;
    Ok(())
}

#[test]
fn append() -> Result<(), FtpError> {
    lock_server();
    let mut client = Client::connect(&get_local_server_hostname(), "user", "user")?;
    let file_data = b"Some data for you";
    let file_name = "readyou.txt";
    client.append(file_name, file_data)?;

    Ok(())
}

#[test]
fn rename_file() -> Result<(), FtpError> {
    lock_server();
    let mut client = Client::connect(&get_local_server_hostname(), "user", "user")?;
    if !client.list_names("/")?.contains(&"testfile".to_string()) {
        client.store("testfile", b"DATA")?;
    }
    client.rename_file("testfile", "testfile.txt")?;

    Ok(())
}

#[test]
fn delete_file() -> Result<(), FtpError> {
    lock_server();
    let mut client = Client::connect(&get_local_server_hostname(), "user", "user")?;
    if !client.list_names("/")?.contains(&"testfile".to_string()) {
        client.store("testfile", b"DATA")?;
    }
    client.delete_file("testfile")?;

    Ok(())
}

#[test]
fn create_directory() -> Result<(), FtpError> {
    lock_server();
    let mut client = Client::connect_with_port(&get_local_server_hostname(), 21, "user", "user")?;
    if client.list_names("/")?.contains(&"new_dir".to_string()) {
        client.remove_directory("new_dir")?;
    }
    client.make_directory("new_dir")
}

#[test]
fn delete_directory() -> Result<(), FtpError> {
    lock_server();
    let mut client = Client::connect(&get_local_server_hostname(), "user", "user")?;
    if !client.list_names("/")?.contains(&"new_dir".to_string()) {
        client.make_directory("new_dir")?;
    }
    client.remove_directory("new_dir")
}

/// Get the hostname for the local server.
fn get_local_server_hostname() -> String {
    std::env::var("SERVER_HOSTNAME").unwrap()
}

static SERVER_MUTEX: OnceCell<Mutex<()>> = OnceCell::new();
/// Tests using the local server can not run concurrently.
fn lock_server() {
    let mutex = SERVER_MUTEX.get_or_init(|| Mutex::new(()));
    let _guard = mutex.lock().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500));
}
