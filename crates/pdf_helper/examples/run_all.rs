use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    pdf_helper::examples::run_all::run()
}
