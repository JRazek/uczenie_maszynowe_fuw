#![feature(iter_array_chunks)]
#![feature(ascii_char)]

use dfdx::prelude::*;

use rand::rngs::StdRng;
use rand::SeedableRng;

use std::path::Path;

use rand::seq::SliceRandom;

use num::FromPrimitive;

use uczenie_maszynowe_fuw::emnist::*;
use uczenie_maszynowe_fuw::emnist_loader::*;
use uczenie_maszynowe_fuw::plots::*;

use plotters::prelude::*;

const LABELS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone, Sequential, Default)]
struct Model<const N_IN: usize, const N_OUT: usize> {
    linear1: LinearConstConfig<N_IN, 128>,
    activation1: Tanh,

    linear2: LinearConstConfig<128, 128>,
    activation2: Tanh,

    linear3: LinearConstConfig<128, 128>,
    activation3: FastGeLU,

    linear7: LinearConstConfig<128, 128>,
    activation7: Tanh,

    linear8: LinearConstConfig<128, 128>,
    activation8: Tanh,

    linear9: LinearConstConfig<128, N_OUT>,
}

fn load_npz_test<const N: usize, const N_IN: usize, E, D>(
    path: &str,
    dev: &D,
) -> Result<Tensor<(usize, Const<N_IN>), E, D>, Box<dyn std::error::Error>>
where
    E: Dtype + FromPrimitive + npyz::Deserialize,
    D: Device<E> + TensorFromVec<E>,
{
    let raw = std::fs::read(path)?;

    let npz = npyz::NpyFile::new(&raw[..])?;

    let n_digits = npz.shape()[0];

    let data = npz.into_vec::<E>()?;

    let tensor = dev.tensor_from_vec(data, (n_digits as usize, Const::<N_IN>::default()));

    Ok(tensor)
}

fn decode_characters_npz<const N_IN: usize, const N_OUT: usize, E, D, M>(
    model: &mut M,
    tensor: Tensor<(usize, Const<N_IN>), E, D>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    E: Dtype,
    D: Device<E>,
    M: Module<Tensor<(usize, Const<N_IN>), E, D>, Output = Tensor<(usize, Const<N_OUT>), E, D>>
        + UpdateParams<E, D>,
{
    let out = model.try_forward(tensor)?;
    let categories = convert_max_outputs_to_category(out)?;

    Ok(categories)
}

//for epoch in 0..10000 {
//    mnist_train.shuffle(&mut rng);
//    let batch_iter = load_chunked_mnist_images::<N_IN, _, _>(&dev, &mnist_train, BATCH_SIZE);

//    for (batch, labels) in batch_iter.clone() {
//        let tensor = batch.try_realize::<Rank2<BATCH_SIZE, N_IN>>();
//        let one_hots = make_one_hots::<BATCH_SIZE, N_OUT, _, _>(&dev, &labels);
//        if let (Ok(tensor), Ok(one_hots)) = (tensor, one_hots) {
//            assert_eq!(labels.len(), BATCH_SIZE);

//            let output = model.try_forward(tensor.retaped::<OwnedTape<_, _>>())?;

//            let loss = cross_entropy_with_logits_loss(output, one_hots);

//            losses.push(loss.as_vec()[0]);

//            let grads = loss.backward();
//            let tensor_grad_magnitude = grads
//                .get(&tensor)
//                .select(dev.tensor(0))
//                .square()
//                .sum()
//                .sqrt();
//            grad_magnitudes.push(tensor_grad_magnitude.as_vec()[0]);

//            rms_prop.update(&mut model, &grads)?;
//        }
//    }

//    model.save_safetensors(model_path)?;

//    let svg_backend =
//        SVGBackend::new("plots/emnist_digits.svg", (1800, 600)).into_drawing_area();

//    let predicted_eval =
//        convert_max_outputs_to_category(model.try_forward(eval_data.clone())?)?;

//    let accuracy =
//        predicted_eval
//            .iter()
//            .zip(eval_labels.iter())
//            .fold(
//                0,
//                |acc, (&predicted, &expected)| {
//                    if predicted == expected {
//                        acc + 1
//                    } else {
//                        acc
//                    }
//                },
//            ) as f32
//            / eval_labels.len() as f32;

//    println!(
//        "Epoch: {}, loss_train: {}, accuracy: {:.2}%",
//        epoch,
//        losses.last().unwrap(),
//        accuracy * 100f32
//    );

//    match svg_backend.split_evenly((1, 3)).as_slice() {
//        [error_matrix_area, losses_area, gradients_area, ..] => {
//            plot_error_matrix(
//                &eval_labels,
//                &predicted_eval,
//                36,
//                &|idx| LABELS.as_ascii().unwrap()[idx].to_string(),
//                &error_matrix_area,
//            )?;

//            plot_log_scale_data(&losses, "loss train", &losses_area)?;
//            plot_log_scale_data(&grad_magnitudes, "gradient norm", &gradients_area)?;
//        }
//        _ => panic!(),
//    }

//    svg_backend.present()?;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev = AutoDevice::default();
    let mut rng = StdRng::seed_from_u64(0);

    let mnist_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data/emnist/".to_string());

    let dev = AutoDevice::default();
    let model_path = Path::new("models/character_recognition");

    println!("Device: {:?}", dev);

    println!("Loading mnist train...");
    let mut mnist_train: Vec<_> = load_data::<f32, _, _>(
        format!("{}/emnist-balanced-train-images-idx3-ubyte.gz", mnist_path),
        format!("{}/emnist-balanced-train-labels-idx1-ubyte.gz", mnist_path),
    )?;
    mnist_train.shuffle(&mut rng);
    mnist_train = mnist_train
        .into_iter()
        .filter(|img| img.classification < LABELS.len() as u8)
        .take(6000)
        .collect();
    println!("Loaded {} training images", mnist_train.len());

    println!("Loading mnist test...");
    let mut mnist_test: Vec<_> = load_data::<f32, _, _>(
        format!("{}/emnist-balanced-test-images-idx3-ubyte.gz", mnist_path),
        format!("{}/emnist-balanced-test-labels-idx1-ubyte.gz", mnist_path),
    )?;
    mnist_test.shuffle(&mut rng);
    mnist_test = mnist_test
        .into_iter()
        .filter(|img| img.classification < LABELS.len() as u8)
        .take(1000)
        .collect();

    println!("Loaded {} test images", mnist_train.len());

    let train_setup = TrainSetup {
        mnist_train,
        mnist_test,
        rng,
    };

    train();
    Ok(())
}
