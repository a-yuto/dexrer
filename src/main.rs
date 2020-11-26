extern crate regex;
use regex::Regex;

fn print_typename<T>(_: T) {
    println!("{}", std::any::type_name::<T>());
}

fn put_conma_vec(_vec: &Vec<String>) -> String {
  let mut put_conma_colums: String = _vec.iter()
                               .map(|s| format!(",{}",s))
                               .collect();
  put_conma_colums.remove(0);
  put_conma_colums
}

fn get_pk_colum(_query: &str) -> String {
  // クエリからは空白とコンマが取り除かれている前提
  let _query_vec:Vec<&str> =  _query.split("\n").collect();
  let _pk = Regex::new(r"#PK").unwrap();
  let _pk_colums:Vec<String> = _query_vec
                            .iter()
                            .filter_map(|str| 
                              if str.contains("PK"){
                                let tmp;
                                match str.contains("AS") {
                                  true => {
                                    tmp = _pk.replace_all(str,"");
                                    Some(tmp.chars().skip(tmp.find("AS")? + 2).collect())
                                  },
                                  false => Some(_pk.replace_all(str,""))
                                }
                              }
                              else {None})
                            .collect();
  // 取得したカラム名をSQLに埋め込めるように,を入れる
  put_conma_vec(&_pk_colums)
}
fn get_table_name(_query: &str) -> Vec<String> {
  // クエリからは空白とコンマが取り除かれている前提
  // テーブル名の取得
  let _with = Regex::new(r"With").unwrap();
  let _as_maru = Regex::new(r"AS\(").unwrap();
  let _query_vec:Vec<&str> =  _query.split("\n").collect();
  let _table_name:Vec<String> = _query_vec
                            .iter()
                            .filter_map(|str| 
                            if str.contains("AS("){
                              Some(_as_maru.replace_all(&_with.replace_all(str,""),""))
                            }
                            else {
                              None
                            })
                            .collect();
  _table_name
}
fn make_pk_check_query(_query :&str) -> String {
  let _space = Regex::new(r" ").unwrap();
  let _conma = Regex::new(r",").unwrap();
  let _cleaned_query = _conma.replace_all(&_space.replace_all(_query,""),"");
  let _put_conma_colums: String = get_pk_colum(&_cleaned_query);
  let _table_name = get_table_name(&_cleaned_query);
  format!(
    "SELECT COUNT(1) FROM {} GROUP BY {} HAVING COUNT(1) <> 1", 
    _table_name[0], 
    _put_conma_colums
  )
}
fn separate_with(_query: &str) -> Vec<String> {
  let mut cnt = 0;
  let mut query = String::new();
  for ch in _query.chars() {
    query.push(ch);
    if ch == '(' {
      cnt += 1;
    }
    if ch == ')' {
      cnt += -1;
      // cntが0になったタイミングで区切りたい
      // -1して0になったタイミングで取得すればそこだけ取れる
      if cnt == 0 {
        query.push('@');
      }
    }
  }
  query.split("@")
       .collect::<Vec<&str>>()
       .iter()
       .filter_map(|query| if query.contains("SELECT"){Some(query.to_string())} else {None})
       .collect::<Vec<String>>() 
}
fn main() {
  let _query = "
  With test AS (
    SELECT
      a AS huga, # PK
      b, # PK
      c
    FROM
      table
  )
  ,test2 AS (
    SELECT
      a, # PK
      b, # PK
      c
    FROM
      table2
  )";
  let separated_queries = separate_with(_query);
  for tmp_query in separated_queries {
    if tmp_query.len() != 0{println!("{:?}",make_pk_check_query(&tmp_query))};
  }
}

#[cfg(test)]
mod test{
  use super::*;
  #[test]
  fn separate_with_works() {
  let _space = Regex::new(r" ").unwrap();
  let _query = 
   _space.replace_all(
  "With test AS (
    SELECT
      a, # PK
      b, # PK
      c
    FROM
      table
  ),test2 AS (
    SELECT
      a, # PK
      b, # PK
      c
    FROM
      table2
  )","");
  let separeted = vec![
  String::from(
  _space.replace_all(
  "With test AS (
    SELECT
      a, # PK
      b, # PK
      c
    FROM
      table
  )","")),
  String::from(
  _space.replace_all(
  ",test2 AS (
    SELECT
      a, # PK
      b, # PK
      c
    FROM
      table2
  )",""))
  ];
  assert_eq!(separeted,separate_with(&_query))
  }
  #[test]
  fn make_pk_check_query_works() {
    let query = "
    With test AS (
      SELECT
        a, # PK
        b, # PK
        c
      FROM
        table
    )
    ";
    let check_query = String::from("SELECT COUNT(1) FROM test GROUP BY a,b HAVING COUNT(1) <> 1");
    assert_eq!(make_pk_check_query(query) , check_query);
  }
  #[test]
  fn withput_wiht_querymake_pk_check_query_works() {
    let query = "
    ,test AS (
      SELECT
        a, # PK
        b, # PK
        c
      FROM
        table
    )
    ";
    let check_query = String::from("SELECT COUNT(1) FROM test GROUP BY a,b HAVING COUNT(1) <> 1");
    assert_eq!(make_pk_check_query(query) , check_query);
  }
  #[test]
  fn put_conma_vec_works() {
    assert_eq!(put_conma_vec(&vec![String::from("a"),String::from("b")]),String::from("a,b"));
  }
  #[test]
  fn get_pk_colum_works() {
    let _query = "
    With test AS (
      SELECT
        a AS hoge, # PK
        b, # PK
        c
      FROM
        table
    )
    ";
    let _space = Regex::new(r" ").unwrap();
    let _conma = Regex::new(r",").unwrap();
    assert_eq!(get_pk_colum(&_conma.replace_all(&_space.replace_all(_query,""),"")),String::from("hoge,b"));
  }
  #[test]
  fn get_table_name_works() {
    let _query = "
    With test AS (
      SELECT
        a, # PK
        b, # PK
        c
      FROM
        table
    )
    ";
    let _space = Regex::new(r" ").unwrap();
    let _conma = Regex::new(r",").unwrap();
    assert_eq!(get_table_name(&_conma.replace_all(&_space.replace_all(_query,""),"")),vec![String::from("test")]);
  }
  #[test]
  fn without_with_get_table_name_works() {
    let _query = "
    ,test AS (
      SELECT
        a, # PK
        b, # PK
        c
      FROM
        table
    )
    ";
    let _space = Regex::new(r" ").unwrap();
    let _conma = Regex::new(r",").unwrap();
    assert_eq!(get_table_name(&_conma.replace_all(&_space.replace_all(_query,""),"")),vec![String::from("test")]);
  }
}
