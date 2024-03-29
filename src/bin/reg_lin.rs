use std::ops::Mul;

use dfdx::tensor::AutoDevice;
use nalgebra::allocator::Allocator;
use nalgebra::{ComplexField, DefaultAllocator, Dim, DimMin, DimMinimum, Matrix, Storage, Vector};
use ndarray::{Array, Dimension};
use plotters::prelude::*;

use rand::distributions::Standard;
use rand::prelude::*;
use rand::rngs::OsRng;

use nalgebra::Vector4;

use dfdx::tensor::TensorFrom;

use dfdx::nn::LinearConstConfig;

use dfdx::nn::BuildModuleExt;
use dfdx::nn::Module;
use dfdx::nn::Sgd;
use dfdx::nn::ZeroGrads;
use dfdx::prelude::*;
use dfdx::tensor::Trace;

pub fn stochastic_gradient_descent() {
    let dev = dfdx::tensor::AutoDevice::default();

    let model = LinearConstConfig::<1, 1>::default();

    let mut module = dev.build_module::<f32>(model);

    let current_model = [0., 1., 2., 3.];
    let voltage_model = [0., 2., 4., 6.];

    let mut grads = module.alloc_grads();

    let mut sgd = Sgd::new(
        &module,
        SgdConfig {
            lr: 0.01,
            momentum: None,
            weight_decay: None,
        },
    );

    for _ in 0..100 {
        voltage_model
            .into_iter()
            .zip(current_model)
            .for_each(|(u, i)| {
                let i_tensor = dev.tensor([i]);

                let u_predicted = module.forward(i_tensor.traced(grads.clone()));
                let loss = (u_predicted - u).square().mean();

                grads = loss.backward();

                sgd.update(&mut module, &grads).unwrap();

                module.zero_grads(&mut grads)
            });
    }

    println!(
        "R: {}, bias: {}",
        module.weight.data().unwrap().get(0).unwrap(),
        module.bias.data().unwrap().get(0).unwrap()
    );
}

pub fn normal_equations<
    T: nalgebra::ComplexField,
    N: nalgebra::DimMin<M>,
    M: nalgebra::Dim,
    S: nalgebra::Storage<T, N, M> + nalgebra::Storage<T, N>,
>(
    features: nalgebra::Matrix<T, N, M, S>,
    y: nalgebra::Vector<T, N, S>,
) -> Option<
    nalgebra::Vector<
        T,
        M,
        <nalgebra::DefaultAllocator as nalgebra::allocator::Allocator<T, M>>::Buffer,
    >,
>
where
    nalgebra::DefaultAllocator: Allocator<T, N, M>
        + Allocator<T, M, N>
        + Allocator<T, M, M>
        + Allocator<T, M>
        + Allocator<(usize, usize), nalgebra::DimMinimum<N, M>>,
    S: Clone,
{
    let x = features
        .transpose()
        .mul(features.clone())
        .try_inverse()?
        .mul(features.transpose())
        .mul(y);

    Some(x)
}

pub fn plot<B>(mut line_plot: ChartBuilder<B>, mut error_histogram_plot: ChartBuilder<B>)
where
    B: DrawingBackend,
{
    let theta0: f64 = 1.;
    let theta1: f64 = 3.;

    let points_n = 100;

    let x = Array::linspace(0., 10., points_n);
    let y_modeled: Array<_, _> = x.iter().map(|&x| theta0 + x * theta1).collect();
    let y = y_modeled
        .iter()
        .map(|&y| y + (OsRng.sample::<f64, _>(Standard)) - 0.5)
        .collect::<Vec<_>>();

    let line_series = LineSeries::new(
        x.iter()
            .cloned()
            .zip(y_modeled.clone())
            .map(|(x, y)| (x, y)),
        BLUE.stroke_width(2),
    )
    .point_size(1);

    let mut line_context = line_plot
        .caption("function", &BLACK)
        .set_all_label_area_size(50)
        .margin(10)
        .build_cartesian_2d(0f64..10f64, 0f64..32f64)
        .unwrap();

    line_context
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .draw()
        .unwrap();

    line_context
        .draw_series(line_series)
        .unwrap()
        .label("modeled")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));

    line_context
        .draw_series(
            x.iter()
                .cloned()
                .zip(y.clone())
                .map(|(x, y)| Circle::new((x, y), 2, RED.filled())),
        )
        .unwrap()
        .label("data")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    let mut histogram_context = error_histogram_plot
        .caption("error: |y - y^|", &BLACK)
        .set_all_label_area_size(50)
        .margin(10)
        .build_cartesian_2d(0usize..y_modeled.len(), 0f64..1f64)
        .unwrap();

    histogram_context
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .draw()
        .unwrap();

    histogram_context
        .draw_series(
            Histogram::vertical(&histogram_context)
                .style(RED.mix(0.5).filled())
                .margin(0) //distance between bars
                .data(
                    y.iter()
                        .cloned()
                        .zip(y_modeled)
                        .enumerate()
                        .map(|(i, (y, y_modeled))| (i, (y - y_modeled).abs())),
                ),
        )
        .unwrap();

    line_context
        .configure_series_labels()
        .border_style(BLACK)
        .background_style(WHITE)
        .draw()
        .unwrap();
}

fn main() {
    let drawing_area = SVGBackend::new("./plots/lin_reg.svg", (800, 600)).into_drawing_area();

    let (left, right) = drawing_area.split_horizontally(400);

    plot(ChartBuilder::on(&left), ChartBuilder::on(&right));

    drawing_area.present().unwrap();

    {
        let currents = Vector4::new(0., 1., 2., 3.);
        let voltage = Vector4::new(0.1, 2.1, 4.3, 6.4);

        let r = normal_equations(currents, voltage).unwrap();
        println!("fitted R: {r}");
    };

    stochastic_gradient_descent();
}
