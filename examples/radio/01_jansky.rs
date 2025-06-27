fn main() {
    let l1 = f32::Length::new::<meter>(100.0);

    println!(
        "{} = {}",
        l1.into_format_args(meter, Abbreviation),
        l1.into_format_args(foot, Abbreviation)
    );
}
