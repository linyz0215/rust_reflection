use test_rust::Matrix;



pub fn main() {
    let a = Matrix::new([1, 2, 3, 4], 2, 2);
    let b = Matrix::new([1, 2, 3, 4], 2, 2);
    let matrix_c = a * b;
    println!("{}", matrix_c);
}