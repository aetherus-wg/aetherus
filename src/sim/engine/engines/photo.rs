//! Photography photon-lifetime engine function.

use crate::{
    img::Colour,
    phys::Photon,
    io::output::{Output, OutputParameter},
    sim::{
        peel_off::peel_off, scatter::scatter, surface::surface, travel::travel, Event, Frame,
        Input,
    },
};
use rand::{rngs::ThreadRng, Rng};

/// Photograph the life of a single photon.
#[allow(clippy::expect_used)]
#[inline]
pub fn photo(
    frames: &[Frame],
    input: &Input,
    mut data: &mut Output,
    mut rng: &mut ThreadRng,
    mut phot: Photon,
) {
    // Add to the emission variables in which the photon is present. 
    for vol in data.get_volumes_for_param_mut(OutputParameter::Emission) {
        if let Some(index) = vol.gen_index(phot.ray().pos()) {
            vol.data_mut()[index] += phot.power() * phot.weight();
        }
    }

    // Common constants.
    let bump_dist = input.sett.bump_dist();
    let loop_limit = input.sett.loop_limit();
    let min_weight = input.sett.min_weight();
    let roulette_barrels = input.sett.roulette_barrels() as f64;
    let roulette_survive_prob = 1.0 / roulette_barrels;

    // Initialisation.
    let phot_col = wavelength_to_rbg(phot.wavelength());
    let mat = input.light.mat();
    let mut env = mat.sample_environment(phot.wavelength());

    // Main event loop.
    let mut num_loops = 0;
    while input.bound.contains(phot.ray().pos()) {
        // Loop limit check.
        if num_loops >= loop_limit {
            println!("[WARN] : Terminating photon: loop limit reached.");
            break;
        }
        num_loops += 1;

        // Roulette.
        if phot.weight() < min_weight {
            let r = rng.gen::<f64>();
            if r > roulette_survive_prob {
                break;
            }
            *phot.weight_mut() *= roulette_barrels;
        }

        // Interaction distances.
        let voxel_dist = data.voxel_dist(&phot);
        let scat_dist = -(rng.gen::<f64>()).ln() / env.inter_coeff();
        let surf_hit = input
            .tree
            .scan(phot.ray().clone(), bump_dist, voxel_dist.min(scat_dist));
        let boundary_hit = input.bound.dist_boundary(phot.ray()).expect("Photon not contained in boundary. ");

        // Event handling.
        match Event::new(voxel_dist, scat_dist, surf_hit, boundary_hit, bump_dist) {
            Event::Voxel(dist) => travel(&mut data, &mut phot, &env, dist + bump_dist),
            Event::Scattering(dist) => {
                travel(&mut data, &mut phot, &env, dist);

                // Capture.
                for (frame, photo) in frames.iter().zip(data.photos.iter_mut()) {
                    if let Some([x, y]) = frame.transform(phot.ray().pos()) {
                        if let Some(weight) = peel_off(input, phot.clone(), &env, *frame.pos()) {
                            photo.pixels_mut()[[x, y]] +=
                                Colour::new(
                                    phot_col[0] as f32,
                                    phot_col[1] as f32,
                                    phot_col[2] as f32,
                                    1.0,
                                ) * (phot.power() * phot.weight() * weight) as f32;
                        }
                    };
                }

                scatter(&mut rng, &mut phot, &env);
            }
            Event::Surface(hit) => {
                travel(&mut data, &mut phot, &env, hit.dist());
                surface(&mut rng, &hit, &mut phot, &mut env, &mut data);
                travel(&mut data, &mut phot, &env, bump_dist);
            },
            Event::Boundary(boundary_hit) => {
                travel(&mut data, &mut phot, &env, boundary_hit.dist());
                input.bound.apply(rng, &boundary_hit, &mut phot);
                // Allow for the possibility that the photon got killed at the boundary - hence don't evolve. 
                if phot.weight() > 0.0 {
                    travel(&mut data, &mut phot, &env, bump_dist);
                }
            }
        }

        if phot.weight() <= 0.0 {
            break;
        }
    }
}

/// Generate the RGB components of a given wavelength.
#[inline]
#[must_use]
pub fn wavelength_to_rbg(mut wavelength: f64) -> [f64; 3] {
    let gamma = 0.8;
    wavelength *= 1.0e9;

    if (380.0..440.0).contains(&wavelength) {
        let a = 0.3 + (0.7 * (wavelength - 380.0) / (440.0 - 380.0));
        let r = ((-(wavelength - 440.0) / (440.0 - 380.0)) * a).powf(gamma);
        let g = 0.0;
        let b = a.powf(gamma);
        return [r, g, b];
    } else if (440.0..490.0).contains(&wavelength) {
        let r = 0.0;
        let g = ((wavelength - 440.0) / (490.0 - 440.0)).powf(gamma);
        let b = 1.0;
        return [r, g, b];
    } else if (490.0..510.0).contains(&wavelength) {
        let r = 0.0;
        let g = 1.0;
        let b = (-(wavelength - 510.0) / (510.0 - 490.0)).powf(gamma);
        return [r, g, b];
    } else if (510.0..580.0).contains(&wavelength) {
        let r = ((wavelength - 510.0) / (580.0 - 510.0)).powf(gamma);
        let g = 1.0;
        let b = 0.0;
        return [r, g, b];
    } else if (580.0..645.0).contains(&wavelength) {
        let r = 1.0;
        let g = (-(wavelength - 645.0) / (645.0 - 580.0)).powf(gamma);
        let b = 0.0;
        return [r, g, b];
    } else if (645.0..750.0).contains(&wavelength) {
        let a = 0.3 + (0.7 * (750.0 - wavelength) / (750.0 - 645.0));
        let r = a.powf(gamma);
        let g = 0.0;
        let b = 0.0;
        return [r, g, b];
    }

    [1.0, 0.0, 1.0]
}
