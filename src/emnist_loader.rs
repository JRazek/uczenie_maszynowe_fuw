use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;

use ndarray::Array2;

use flate2::read::GzDecoder;

use std::io::{Cursor, Read};

#[derive(Debug)]
struct MnistData {
    sizes: Vec<i32>,
    data: Vec<u8>,
}

impl MnistData {
    fn new(f: &File) -> Result<MnistData, std::io::Error> {
        let mut gz = GzDecoder::new(f);

        let mut contents: Vec<u8> = Vec::new();
        gz.read_to_end(&mut contents)?;
        let mut r = Cursor::new(&contents);

        let magic_number = r.read_i32::<BigEndian>()?;

        let mut sizes: Vec<i32> = Vec::new();
        let mut data: Vec<u8> = Vec::new();

        match magic_number {
            2049 => {
                sizes.push(r.read_i32::<BigEndian>()?);
            }
            2051 => {
                sizes.push(r.read_i32::<BigEndian>()?);
                sizes.push(r.read_i32::<BigEndian>()?);
                sizes.push(r.read_i32::<BigEndian>()?);
            }
            _ => panic!(),
        }

        r.read_to_end(&mut data)?;

        Ok(MnistData { sizes, data })
    }
}

#[derive(Debug)]
pub struct MnistImage<T> {
    pub image: Array2<T>,
    pub classification: u8,
}

pub fn load_data<T>(dataset_name: &str) -> Result<Vec<MnistImage<T>>, std::io::Error>
where
    T: num::Float + num::FromPrimitive,
{
    let filename = format!("{}/emnist-letters-train-labels-idx1-ubyte.gz", dataset_name);
    let label_data = &MnistData::new(&(File::open(filename))?)?;

    let filename = format!("{}/emnist-letters-train-images-idx3-ubyte.gz", dataset_name);
    let images_data = &MnistData::new(&(File::open(filename))?)?;

    let mut images: Vec<Array2<T>> = Vec::new();
    let image_shape = (images_data.sizes[1] * images_data.sizes[2]) as usize;

    for i in 0..images_data.sizes[0] as usize {
        let start = i * image_shape;
        let image_data = images_data.data[start..start + image_shape].to_vec();
        let image_data: Vec<T> = image_data
            .into_iter()
            .map(|x| T::from_u8(x).unwrap() / T::from_u8(255).unwrap())
            .collect();
        images.push(Array2::from_shape_vec((image_shape, 1), image_data).unwrap());
    }

    let classifications: Vec<u8> = label_data.data.clone();

    let mut ret: Vec<MnistImage<T>> = Vec::new();

    for (image, classification) in images.into_iter().zip(classifications.into_iter()) {
        ret.push(MnistImage {
            image,
            classification,
        })
    }

    Ok(ret)
}
