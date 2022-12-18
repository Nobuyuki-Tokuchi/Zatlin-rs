
#[cfg(feature="use_macro")]
mod macro_test {
    use zatlin_internal::error::ErrorValue;
    use zatlin_rs::Zatlin;

    use zatlin_macro::zatlin;
    use zatlin_internal::ZatlinData;

    #[test]
    fn macro_test() {
        let data: Result<ZatlinData, ErrorValue> = zatlin!{
            Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
            Ce = "" | "b" | "d" | "g" | "m" | "n" | "h";
            
            Va = "a" | "á" | "à" | "ä";
            Ve = "e" | "é" | "è" | "ë";
            Vi = "i" | "í" | "ì" | "ï";
            Vo = "o" | "ó" | "ò" | "ö";
            Vu = "u" | "ú" | "ù" | "ü";
            Vy = "y" | "ý" | "ỳ" | "ÿ";
            
            Vxi = Va "i" | Ve "i" | Vo "i" | Vi "a" | Vi "e";
            Vxu = Va "u" | Vo "u" | Vu "e" | Vu "i";
            Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;
            
            % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ "y" | ^ "ý" | ^ "ỳ" | ^ "ÿ" | ^ "wu" | ^ "wú" | ^ "wù" | ^ "wü" | ^ "hy" | ^ "hý" | ^ "hỳ" | ^ "hÿ" | ^ "qy" | ^ "qý" | ^ "qỳ" | ^ "qÿ" | ^ "ry" | ^ "rý" | ^ "rỳ" | ^ "rÿ" | ^ "ny" | ^ "ný" | ^ "nỳ" | ^ "nÿ" | ^ "my" | ^ "mý" | ^ "mỳ" | ^ "mÿ";
        };
    
        let data = match data {
            Ok(result) => result,
            Err(error) => {
                println!("{}", error);
                assert!(false);
                return;
            }
        };
    
        let generator = Zatlin::default();
        let result = generator.generate_many_by(&data, 10);
        
        for item in result.iter() {
            match item {
                Ok(value) => {
                    print!("{} ", value);
                },
                Err(message) => {
                    print!("({}) ", message);
                },
            }
        }
        println!("");
        assert!(result.iter().all(|x| x.is_ok()));
    }


    #[cfg(feature="use_macro")]
    #[test]
    fn unofficial_circ_macro() {
        let data: Result<ZatlinData, ErrorValue> = zatlin!{
            Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
            Ce = "" | "b" | "d" | "g" | "m" | "n" | "h";
            
            Va = "a" | "á" | "à" | "ä";
            Ve = "e" | "é" | "è" | "ë";
            Vi = "i" | "í" | "ì" | "ï";
            Vo = "o" | "ó" | "ò" | "ö";
            Vu = "u" | "ú" | "ù" | "ü";
            Vy = "y" | "ý" | "ỳ" | "ÿ";
            
            Vxi = (Va | Ve | Vo) "i" | Vi ( "a" | "e" );
            Vxu = ( Va | Vo ) "u" | Vu ("e" | "i");
            Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;
            % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ ("" | "w" | "h" | "q" | "r" | "n" | "m") ("y" | "ý" | "ỳ" | "ÿ");
        };

        let data = match data {
            Ok(result) => result,
            Err(error) => {
                println!("{}", error);
                assert!(false);
                return;
            }
        };

        let generator = Zatlin::default();
        let result = generator.generate_many_by(&data, 32);
        
        for item in result.iter() {
            match item {
                Ok(value) => {
                    print!("{} ", value);
                },
                Err(message) => {
                    print!("({}) ", message);
                },
            }
        }
        println!("");
        assert!(result.iter().all(|x| x.is_ok()));
    }

    #[cfg(feature="use_macro")]
    #[test]
    fn unofficial_destruct_pattern_macro() {
        let data: Result<ZatlinData, ErrorValue> = zatlin!{
            Ca = "p" | "b" | "f" | "v" | "m" | "t" | "d" | "s" | "z" | "n";
            Cb = "p" | "b" | "f" | "v" | "m" | "k" | "g" | "h";
            C = Ca | Cb;
            Vi = "a" | "e" | "i";
            Vu = "a" | "o" | "u";
            V = Vi | Vu;
    
            X : Vx <- V = C Vx C Vx;
            Y : Vx <- V, Cx <- C = Vx Cx Vx Cx | Cx Vx Cx Vx Cx;
            % V | V C | C V | C V C | X;
        };

        let data = match data {
            Ok(result) => result,
            Err(error) => {
                println!("{}", error);
                assert!(false);
                return;
            }
        };

        let generator = Zatlin::default();
        let result = generator.generate_many_by(&data, 32);
        
        for item in result.iter() {
            match item {
                Ok(value) => {
                    print!("{} ", value);
                },
                Err(message) => {
                    print!("({}) ", message);
                },
            }
        }
        println!("");
        assert!(result.iter().all(|x| x.is_ok()));
    }
}