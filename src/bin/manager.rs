// ToDo: parse a cargo.toml file and check for table consistency, nesting and so on
// ToDo: keys cannot be defined multiple times
// ToDo: cannot define a table more than once
// ToDo: The top-level table, also called the root table, starts at the beginning of the document and ends just before the first table header (or EOF). Unlike other tables, it is nameless and cannot be relocated.
// ToDo: # THE FOLLOWING IS INVALID
//
// # This defines the value of fruit.apple to be an integer.
// fruit.apple = 1
//
// # But then this treats fruit.apple like it's a table.
// # You can't turn an integer into a table.
// fruit.apple.smooth = true

// ToDo: come up for a proper data structure for checking consistency
// HashMaps will be too complicated and probably inefficient
// a tree of some kind has to work
// fn verify_consistency(tab_vec: Vec<Table>) -> Result<HashMap<String, Table>, ()> {
//     let mut table_map: HashMap<String, Table> = HashMap::new();
//     for table in tab_vec {
//         let tabs: Vec<&str> = table.header.split(",").collect();
//         for tab in tabs {
//             table_map.insert(tab.to_string(), Table {header: tab.to_string(), key_val_vec: vec![]})
//         }
//
//
//     }
//
//     Ok(table_map)
// }

fn main() {}
