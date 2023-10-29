# Zatlin
## 概要
これは[Zatlin](https://github.com/Ziphil/ZatlinTypescript)のRust移植版のライブラリです．
現時点ではVer 1.1をある程度実装しております．

## 本家と異なる部分
把握している限りの相違部分は以下のものとなります．
* エラーの出力異なる
* リトライ方法が異なる

## zatlinマクロ
zatlinマクロを使用することで，Rustでzatlin構文をほぼそのままで記述することが可能となります．
```rust
let data = zatlin! {
    Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m"
    Ce = "" | "b" | "d" | "g" | "m" | "n" | "h"

    Va = "a" | "á" | "à" | "ä";
    Ve = "e" | "é" | "è" | "ë";
    Vi = "i" | "í" | "ì" | "ï";
    Vo = "o" | "ó" | "ò" | "ö";
    Vu = "u" | "ú" | "ù" | "ü";
    Vy = "y" | "ý" | "ỳ" | "ÿ";

    Vxi = (Va | Ve | Vo) "i" | Vi ("a" | "e")
    Vxu = (Va | Vo) "u" | Vu ("e" | "i")
    Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu

    % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Vy | ^ "w" Vu | ^ ("h" | "q" | "r" | "n" | "m") Vy;
}?;
let result = zatlin::generate_by(&data)?;
```
マクロを使用する場合には，featuresにて"use_macro"を指定する必要があります．
```toml
[dependencies]
zatlin = { version = "0.2", features = [ "use_macro" ] }
```

### マクロの制限
* 行末でのセミコロンの省略ができない．