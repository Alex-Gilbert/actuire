use super::hotel_data::Hotel;

struct Player {
    name: String,
    stocks: [usize; Hotel::count()],
    cash: usize,
}
