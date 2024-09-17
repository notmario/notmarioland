#[macro_export]
macro_rules! texture {
	(
		$textures: expr
		$(, $paths:expr)+
	) => {
		{
			let mut texture = None;
			let paths = [$($paths, )+];
			// let skins = std::fs::read_to_string("enabledskins").expect("should exist if all is fine");
			// let skins: Vec<&str> = skins.lines().collect();
			for path in paths {
				if $textures.contains_key(path) {
					texture = Some($textures.get(path).expect("it exists").clone());
					break
				} else {

          let t = load_texture(path).await.unwrap();
          $textures.insert(path.to_string(), t.clone());
					t.set_filter(FilterMode::Nearest);
          texture = Some(t);

					// println!("loading {} for the first time!", path);
					// std::thread::sleep(std::time::Duration::from_millis(16));
				}
			}
			if texture.is_none() {
				panic!("Could not load texture {}",paths.last().expect("there will be a last one"))
			}
			texture.expect("it is not none")
		}	};
}

#[macro_export]
macro_rules! texture_cache {
	(
		$textures: expr
		$(, $paths:expr)+
	) => {
		{
			let mut texture = None;
			let paths = [$($paths, )+];
			// let skins = std::fs::read_to_string("enabledskins").expect("should exist if all is fine");
			// let skins: Vec<&str> = skins.lines().collect();
			for path in paths {
				if $textures.contains_key(path) {
					texture = Some($textures.get(path).expect("it exists").clone());
					break
				}
			}
			if texture.is_none() {
				panic!("Tried to use cached texture {}, but it was not loaded.",paths.last().expect("there will be a last one"))
			}
			texture.expect("it is not none")
		}
	};
}

#[macro_export]
macro_rules! sound {
	(
		$sounds: expr
		$(, $paths:expr)+
	) => {
		{
			let mut sound = None;
			let paths = [$($paths, )+];
			// let skins = std::fs::read_to_string("enabledskins").expect("should exist if all is fine");
			// let skins: Vec<&str> = skins.lines().collect();
			for path in paths {
				if $sounds.contains_key(path) {
					sound = Some($sounds.get(path).expect("it exists").clone());
					break
				} else {

          let t = load_sound(path).await.unwrap();
          $sounds.insert(path.to_string(), t.clone());
          sound = Some(t);
					// std::thread::sleep(std::time::Duration::from_millis(16));
				}
			}
			if sound.is_none() {
				panic!("Could not load sound {}",paths.last().expect("there will be a last one"))
			}
			sound.expect("it is not none")
		}
		// if !$textures.contains_key($paths) {
		//     let mut texture = match Texture::new($ctx, $path) {
		//         Ok(it) => {$textures.insert($path.to_string(), it.clone()); it},
		//         Err(_) => panic!("Could not load texture {}",$path)
		//     };
		//     texture.set_filter_mode($ctx, tetra::graphics::FilterMode::Nearest);

		//     texture
		// } else {
		//     $textures.get($path).expect("it exists").clone()
		// }
	};
}

#[macro_export]
macro_rules! raw_sound {
    (
		$sounds: expr,
		$ctx: expr,
		// $(, $paths:expr)+
		$path: expr
	) => {
        let d = std::fs::read(&$path).expect("should exist");
        let t = RawSound::load($ctx, &d);
        $sounds.insert($path.to_string(), t);
        // std::thread::sleep(std::time::Duration::from_millis(16));
    };
}

#[macro_export]
macro_rules! sound_cache {
	(
		$sounds: expr
		$(, $paths:expr)+
	) => {
		{
			let mut sound = None;
			let paths = [$($paths, )+];
			// let skins = std::fs::read_to_string("enabledskins").expect("should exist if all is fine");
			// let skins: Vec<&str> = skins.lines().collect();
			for path in paths {
				if $sounds.contains_key(path) {
					sound = Some($sounds.get(path).expect("it exists").clone());
					break
				}			}
			if sound.is_none() {
				panic!("Could not load sound {}",paths.last().expect("there will be a last one"))
			}
			sound.expect("it is not none")
		}
		// if !$textures.contains_key($paths) {
		//     let mut texture = match Texture::new($ctx, $path) {
		//         Ok(it) => {$textures.insert($path.to_string(), it.clone()); it},
		//         Err(_) => panic!("Could not load texture {}",$path)
		//     };
		//     texture.set_filter_mode($ctx, tetra::graphics::FilterMode::Nearest);

		//     texture
		// } else {
		//     $textures.get($path).expect("it exists").clone()
		// }
	};
}
