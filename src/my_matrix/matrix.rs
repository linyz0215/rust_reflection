use std::{fmt::Display, ops::Mul, thread};

use crate::{Vector, dot_vector};
use anyhow::Result;

#[derive(Debug)]
pub struct Matrix {
    pub data: Vector,
    pub row: i32,
    pub col: i32,
}

pub struct MsgInput {
    idx: i32,
    row: Vector,
    col: Vector,
}

pub struct MsgOutput {
    idx: i32,
    value: i32,
}

pub struct Msg {
    sender: oneshot::Sender<MsgOutput>,
    content: MsgInput,
}
impl Matrix {
    pub fn new(data: impl Into<Vec<i32>>, row: i32, col: i32) -> Self {
        Matrix {
            data: Vector::new(data),
            row,
            col,
        }
    }
}

impl MsgInput {
    pub fn new(idx: i32, row: Vector, col: Vector) -> Self {
        MsgInput { idx, row, col }
    }
}

impl MsgOutput {
    pub fn new(idx: i32, value: i32) -> Self {
        MsgOutput { idx, value }
    }
}

impl Msg {
    pub fn new(sender: oneshot::Sender<MsgOutput>, content: MsgInput) -> Self {
        Msg { sender, content }
    }
}
pub fn multiply(matrix_a: &Matrix, matrix_b: &Matrix) -> Result<Matrix> {
    if matrix_a.col != matrix_b.row {
        anyhow::bail!("Incompatible matrix dimensions for multiplication");
    }

    let n = matrix_a.row * matrix_b.col;
    let mut Receiver = Vec::with_capacity(n as usize);
    let mut data = vec![i32::default(); n as usize];
    let (tx, rx) = std::sync::mpsc::channel::<Msg>();
    for i in 0..matrix_a.row {
        for j in 0..matrix_b.col {
            let matrix_a_start = (i * matrix_a.col) as usize;
            let matrix_a_end = matrix_a_start + matrix_a.col as usize;
            let row = Vector::new(&matrix_a.data[matrix_a_start..matrix_a_end]);
            let matrix_b_start = j as usize;
            let col = Vector::new(
                matrix_b
                    .data
                    .iter()
                    .skip(matrix_b_start)
                    .step_by(matrix_b.col as usize)
                    .cloned()
                    .collect::<Vec<i32>>(),
            );
            let msginput = MsgInput::new(i * matrix_b.col + j, row, col);
            let (sender, receiver) = oneshot::channel::<MsgOutput>();
            let msg = Msg::new(sender, msginput);
            tx.send(msg)?;
            Receiver.push(receiver);
        }   
    }

    drop(tx);

    thread::spawn(move || -> Result<()> {
        while let Ok(msg) = rx.recv() {
            let value = dot_vector(&msg.content.row, &msg.content.col);
            let idx = msg.content.idx;
            let output = MsgOutput::new(idx, value);
            msg.sender.send(output)?;
        }
        Ok(())
    });

    for rx in Receiver {
        let output = rx.recv()?;
        data[output.idx as usize] = output.value;
    }

    Ok(Matrix::new(data, matrix_a.row, matrix_b.col))

}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Implement matrix multiplication logic here
        // For simplicity, this is just a placeholder
        multiply(&self, &rhs).expect("multiply error")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, Vector::new([22, 28, 49, 64]));
        //assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, Vector::new([7, 10, 15, 22]));
        //assert_eq!(format!("{:?}", c), "{7 10, 15 22}");
        Ok(())
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for i in 0..self.row {
            let start = (i * self.col) as usize;
            let end = start + self.col as usize;
            s.push_str("[ ");
            let row_str = self.data[start..end].iter().map(|x| x.to_string() + " ").collect::<String>();
            s.push_str(row_str.as_str());
            s.push_str("]");
            if i < self.row - 1 {
                s.push_str(", ");
            }
        }
        write!(f, "{}", s)
    }
}