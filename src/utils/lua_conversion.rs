//! Lua 数据转换工具
//!
//! 提供 Lua 表与 Rust 字节数组之间的转换函数。

use mlua::{Lua, Result as LuaResult, Table};

/// 将 Lua 表（1 索引的字节数组）转换为 Vec<u8>
pub fn lua_table_to_bytes(table: &Table) -> LuaResult<Vec<u8>> {
    let len = table.len().unwrap_or(0) as usize;
    let mut bytes = Vec::with_capacity(len);
    for i in 1..=len {
        let byte: u8 = table.get(i).unwrap_or(0);
        bytes.push(byte);
    }
    Ok(bytes)
}

/// 将字节切片转换为 Lua 表（1 索引数组）
pub fn bytes_to_lua_table<'lua>(lua: &'lua Lua, data: &[u8]) -> LuaResult<Table<'lua>> {
    let table = lua.create_table()?;
    for (i, &byte) in data.iter().enumerate() {
        table.set(i + 1, byte)?;
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_table_to_bytes() {
        let lua = Lua::new();
        let table = lua.create_table().unwrap();
        table.set(1, 0x48u8).unwrap(); // 'H'
        table.set(2, 0x69u8).unwrap(); // 'i'

        let bytes = lua_table_to_bytes(&table).unwrap();
        assert_eq!(bytes, vec![0x48, 0x69]);
    }

    #[test]
    fn test_bytes_to_lua_table() {
        let lua = Lua::new();
        let data = vec![0x48u8, 0x69];

        let table = bytes_to_lua_table(&lua, &data).unwrap();
        let byte1: u8 = table.get(1).unwrap();
        let byte2: u8 = table.get(2).unwrap();

        assert_eq!(byte1, 0x48);
        assert_eq!(byte2, 0x69);
    }
}
