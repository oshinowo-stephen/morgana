use rand::Rng;

pub fn generate_random_number() -> usize {
  let mut generated_number = String::new();

  for _ in 0..16 {
    let i = rand::thread_rng().gen_range(0..9);
    generated_number.push_str(&i.to_string());
  }

  generated_number.parse::<usize>()
    .unwrap_or_default()
}
