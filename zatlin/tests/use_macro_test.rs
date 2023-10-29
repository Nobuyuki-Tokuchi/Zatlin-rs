
#[cfg(feature="use_macro")]
mod macro_test {
    use zatlin::{Error, Data, Zatlin, zatlin};

    #[test]
    fn macro_test() {
        let data: Result<Data, Error> = zatlin!{
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

    #[test]
    fn macro_test2() {
        let data: Result<Data, Error> = zatlin!{
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
            
            % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Vy | ^ "w" Vu | ^ "h" Vy | ^ "q" Vy | ^ "r" Vy | ^ "n" Vy | ^ "m" Vy;
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

    #[test]
    fn macro_test3() {
        let data: Result<Data, Error> = zatlin!{
            Cs = "" | "b" | "p" | "f" | "v" | "d" | "t" | "s" | "z" | "c" | "j" | "g" | "k" | "h" | "q" | "r" | "w" | "n" | "m";
            Ce = "" | "b" | "d" | "g" | "m" | "n" | "h";
            
            Va = "a" | "á" | "à" | "ä";
            Ve = "e" | "é" | "è" | "ë";
            Vi = "i" | "í" | "ì" | "ï";
            Vo = "o" | "ó" | "ò" | "ö";
            Vu = "u" | "ú" | "ù" | "ü";
            Vy = "y" | "ý" | "ỳ" | "ÿ";
            
            Vxi = (Va | Ve | Vo) "i" | Vi ("a" | "e");
            Vxu = (Va | Vo) "u" | Vu ("e" | "i");
            Vx = Va | Ve | Vi | Vo | Vu | Vy | Vxi | Vxu;
            
            % Cs Vx Ce | Cs Vx Ce Cs Vx Ce - ^ Vy | ^ "w" Vu | ^ ("h" | "q" | "r" | "n" | "m") Vy;
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
}