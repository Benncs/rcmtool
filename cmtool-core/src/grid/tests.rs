use super::*;
#[cfg(test)]
mod test {

    use super::*;

    const NUMBER_POINT_AX1: usize = 8;
    const MAX_AX1: f64 = 4.;

    const NUMBER_POINT_AX2: usize = 5;
    const _MAX_AX2: f64 = 2.;

    const NUMBER_POINT_AX3: usize = 10;
    const MAX_AX3: f64 = 10.;

    fn utils_planes_neighbors(
        nax1: usize,
        nax2: usize,
        nax3: usize,
    ) -> (Box<dyn CompartmentMesh>, f64, f64, f64) {
        let ax1 = AxisDescriptor::new(0., MAX_AX1, nax1);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, nax2);

        let ax3 = AxisDescriptor::new(0., MAX_AX3, nax3);

        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let step_r = MAX_AX1 / (nax1 as f64);
        let step_theta = 2.0 * std::f64::consts::PI / (nax2 as f64);
        let step_z = MAX_AX3 / (nax3 as f64);

        (mesh, step_r, step_theta, step_z)
    }

    fn ref_mesh_cyclindrical() -> Box<dyn CompartmentMesh> {
        let ax1 = AxisDescriptor::new(0., MAX_AX1, NUMBER_POINT_AX1);
        let ax2 = AxisDescriptor::new(
            -std::f64::consts::PI,
            std::f64::consts::PI,
            NUMBER_POINT_AX2,
        );
        let ax3 = AxisDescriptor::new(0., MAX_AX3, NUMBER_POINT_AX3);
        get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3])
    }

    #[test]
    fn t_get_mesh() {
        let ax1 = AxisDescriptor::new(0.5, 1., 10);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 20);
        let ax3 = AxisDescriptor::new(0., 2., 15);
        let expected_step_x = 1. / 10.; //Cylindrical start ax from 0 to max_range
        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);
        assert!(
            mesh.mesh_step_axis(0) == expected_step_x,
            "{} {}",
            mesh.mesh_step_axis(0),
            expected_step_x
        );
    }

    #[test]
    fn t_getter() {
        let mesh = ref_mesh_cyclindrical();
        assert!(mesh.max_axis(0) == MAX_AX1);
        assert!(mesh.max_axis(1) == std::f64::consts::PI);
        assert!(mesh.max_axis(2) == MAX_AX3);

        assert!(mesh.min_axis(0) == 0.);
        assert!(mesh.min_axis(1) == -std::f64::consts::PI);

        assert!(mesh.n_points_axis(0) == NUMBER_POINT_AX1);
        assert!(mesh.n_points_axis(1) == NUMBER_POINT_AX2);
        assert!(mesh.n_points_axis(2) == NUMBER_POINT_AX3);
        assert!(mesh.number_cell() == NUMBER_POINT_AX1 * NUMBER_POINT_AX2 * NUMBER_POINT_AX3);
    }

    #[test]
    fn t_cell_surface_cylindrical() {
        let ax1 = AxisDescriptor::new(0., 4., 10);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 10);
        let ax3 = AxisDescriptor::new(0., 2., 10);

        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let r_face_center = 0.4;
        let dtheta = 2. * std::f64::consts::PI / 10.;
        let dz = 2. / 10.;
        let expected_surface = r_face_center * dtheta * dz;
        let actual_surface = mesh.cell_surface(0, OrientedAxis::I);
        assert!(
            (actual_surface - expected_surface).abs() < 1e-10,
            "Radial face surface incorrect: got {}, expected {}",
            actual_surface,
            expected_surface
        );

        let dr = 4. / 10.; // 0.4
        let expected_surface = dr * dz;
        let actual_surface = mesh.cell_surface(0, OrientedAxis::J);
        assert!(
            (actual_surface - expected_surface).abs() < 1e-10,
            "Theta face surface incorrect: got {}, expected {}",
            actual_surface,
            expected_surface
        );

        let r1 = 0.0;
        let r2 = 0.4;
        let dtheta = 2. * std::f64::consts::PI / 10.;

        let expected_surface = 0.5 * (r2 * r2 - r1 * r1) * dtheta;
        let actual_surface = mesh.cell_surface(0, OrientedAxis::K);
        assert!(
            (actual_surface - expected_surface).abs() < 1e-10,
            "Axial face surface incorrect: got {}, expected {}",
            actual_surface,
            expected_surface
        );
    }

    ///Every cell takes its outer edge for the radial face, not only the one touching the axis
    #[test]
    fn t_cell_surface_cylindrical_outer_ring() {
        let n_r = 10;
        let ax1 = AxisDescriptor::new(0., 4., n_r);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 10);
        let ax3 = AxisDescriptor::new(0., 2., 10);

        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let dr = 4. / n_r as f64;
        let dtheta = 2. * std::f64::consts::PI / 10.;
        let dz = 2. / 10.;

        //Third ring, its radial face sits at r = 3 * dr
        let cell_id = mesh.cell_from_ax_points(&[2, 0, 0]).expect("cell");
        let expected_surface = 3. * dr * dtheta * dz;
        let actual_surface = mesh.cell_surface(cell_id, OrientedAxis::I);

        assert!(
            (actual_surface - expected_surface).abs() < 1e-10,
            "Radial face surface incorrect: got {}, expected {}",
            actual_surface,
            expected_surface
        );
    }

    #[test]
    fn t_cell_volume_cylindrical() {
        let ax1 = AxisDescriptor::new(0., 4., 10);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 10);
        let ax3 = AxisDescriptor::new(0., 2., 10);
        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let r1 = 0.0;
        let r2 = 0.4;
        let dtheta = 2. * std::f64::consts::PI / 10.;
        let dz = 2.0 / 10.0;

        let expected_volume = 0.5 * (r2 * r2 - r1 * r1) * dtheta * dz;
        let actual_volume = mesh.cell_volume(0);
        assert!(
            (expected_volume - actual_volume).abs() < 1e-10,
            "Axial face volume incorrect: got {}, expected {}",
            actual_volume,
            expected_volume
        );

        let expected_full_volume = std::f64::consts::PI * 4. * 4. * 2.;
        let full_volume: f64 = (0..mesh.number_cell()).map(|e| mesh.cell_volume(e)).sum();
        assert!(
            (expected_full_volume - full_volume).abs() < 1e-10,
            "full_volume incorrect: got {}, expected {}",
            full_volume,
            expected_full_volume
        );
    }

    #[test]
    fn t_identification_cylindrical() {
        let mesh = ref_mesh_cyclindrical();

        let assert_id = |a: Coords3, expect: usize| {
            let CartesianCoordinates(aa) = CylindricalCoordinates(a).into();

            let id1 = mesh
                .cell_from_coordinates(&aa)
                .expect("Test neighbors: coordinates for cell a are outside the mesh.");

            assert!(
                id1 == expect,
                "Assertion failed: expected {:?}, got {:?}",
                expect,
                id1
            );
        };

        assert_id([0., -std::f64::consts::PI, 0.], 0);

        assert_id([0., -std::f64::consts::PI, MAX_AX3], NUMBER_POINT_AX3 - 1);
        let theta = -std::f64::consts::PI + mesh.mesh_step_axis(1) * 1.1;
        //R!=0 because with cartesian conversion is x=rcos(theta) if theta changes but no r its the same compartment
        assert_id([0.01, theta, 0.], NUMBER_POINT_AX3);
        //-1 because we consider cell ID for 0 to n-1
        assert_id(
            [MAX_AX1, std::f64::consts::PI, MAX_AX3],
            (NUMBER_POINT_AX3 * NUMBER_POINT_AX1 * NUMBER_POINT_AX2) - 1,
        );
    }

    #[test]
    fn t_boundary_cylindrical() {
        let ax1 = AxisDescriptor::new(0., MAX_AX1, 5);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 10);
        let ax3 = AxisDescriptor::new(0., MAX_AX3, 10);
        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let mut v = mesh.get_boundary();
        let w = [
            0, 9, 10, 19, 20, 29, 30, 39, 40, 49, 50, 59, 60, 69, 70, 79, 80, 89, 90, 99, 100, 109,
            110, 119, 120, 129, 130, 139, 140, 149, 150, 159, 160, 169, 170, 179, 180, 189, 190,
            199, 200, 209, 210, 219, 220, 229, 230, 239, 240, 249, 250, 259, 260, 269, 270, 279,
            280, 289, 290, 299, 300, 309, 310, 319, 320, 329, 330, 339, 340, 349, 350, 359, 360,
            369, 370, 379, 380, 389, 390, 399, 400, 401, 402, 403, 404, 405, 406, 407, 408, 409,
            410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426,
            427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439, 440, 441, 442, 443,
            444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460,
            461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477,
            478, 479, 480, 481, 482, 483, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494,
            495, 496, 497, 498, 499,
        ];
        v.sort();
        assert_eq!(v, w);
    }

    ///Three different axis sizes, so that the expected count cannot coincide with a wrong formula
    #[test]
    fn t_boundary_cylindrical_asymmetric() {
        let (n_r, n_theta, n_z) = (3, 4, 5);
        let ax1 = AxisDescriptor::new(0., MAX_AX1, n_r);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, n_theta);
        let ax3 = AxisDescriptor::new(0., MAX_AX3, n_z);
        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);

        let mut v = mesh.get_boundary();

        //Both z faces plus the outer r shell of the remaining slices
        assert_eq!(v.len(), 2 * n_r * n_theta + n_theta * (n_z - 2));
        assert!(v.iter().all(|&cell_id| cell_id < n_r * n_theta * n_z));

        v.sort();
        let n_before_dedup = v.len();
        v.dedup();
        assert_eq!(v.len(), n_before_dedup, "a cell is reported twice");
    }

    #[test]
    fn test_theta_plane_neighbors() {
        let (mesh, step_r, step_theta, step_z) =
            utils_planes_neighbors(NUMBER_POINT_AX1, NUMBER_POINT_AX2, NUMBER_POINT_AX3);
        let eps_step = 0.1;
        let aa = [0.1, -std::f64::consts::PI, 0.0];
        let bb = [0.1, -std::f64::consts::PI + step_theta + eps_step, 0.0];
        let CartesianCoordinates(aa) = CylindricalCoordinates(aa).into();
        let CartesianCoordinates(bb) = CylindricalCoordinates(bb).into();
        let cell1_id = mesh.cell_from_coordinates(&aa).unwrap();
        let cell2_id = mesh.cell_from_coordinates(&bb).unwrap();

        assert!(cell1_id != cell2_id);

        let (plane, axis) = mesh.get_interface_plane(cell1_id, cell2_id);
        assert_eq!(axis, 1);
        assert!((plane.extent_u[0] - 0.0).abs() < 1e-8); // r min
        assert!((plane.extent_u[1] - step_r).abs() < 1e-8); // r max
        assert!((plane.extent_v[0] - 0.0).abs() < 1e-8); // z min
        assert!((plane.extent_v[1] - step_z).abs() < 1e-8); // z max
        // assert!(plane.origin[0]==mesh.ge
    }

    #[test]
    fn test_r_plane_neighbors() {
        let (mesh, step_r, step_theta, step_z) =
            utils_planes_neighbors(NUMBER_POINT_AX1, NUMBER_POINT_AX2, NUMBER_POINT_AX3);
        let eps_step = 0.1;
        let aa = [0.0, 0.0, 0.0];
        let bb = [step_r + eps_step, 0.0, 0.0];

        let CartesianCoordinates(aa) = CylindricalCoordinates(aa).into();
        let CartesianCoordinates(bb) = CylindricalCoordinates(bb).into();

        let cell1_id = mesh.cell_from_coordinates(&aa).unwrap();
        let cell2_id = mesh.cell_from_coordinates(&bb).unwrap();

        assert!(cell1_id != cell2_id);

        let m = mesh.cell_points(cell1_id);
        let mp = mesh.get_cell_edge(1, m[1]);
        let mn = mesh.get_cell_edge(1, m[1] + 1);

        let (plane, axis) = mesh.get_interface_plane(cell1_id, cell2_id);
        assert_eq!(axis, 0);
        assert!(
            (plane.extent_u[0] - mp).abs() < 1e-8,
            "{} vs {}",
            plane.extent_u[0],
            step_theta
        );
        assert!(
            (plane.extent_u[1] - mn).abs() < 1e-8,
            "{} vs {}",
            plane.extent_u[1],
            0.
        );

        assert!((plane.extent_v[0] - 0.0).abs() < 1e-8);
        assert!((plane.extent_v[1] - step_z).abs() < 1e-8);
    }

    #[test]
    fn fuzz_z_plane_neighbors() {
        let nax1 = [5, 7, 10];
        let nax2 = [5, 7, 10];
        let nax3 = [5, 7, 10];

        for &n1 in nax1.iter() {
            for &n2 in nax2.iter() {
                for &n3 in nax3.iter() {
                    eprintln!("{} {} {}", n1, n2, n3);
                    test_z_plane_neighbors(n1, n2, n3);
                }
            }
        }
    }

    fn test_z_plane_neighbors(nax1: usize, nax2: usize, nax3: usize) {
        let (mesh, step_r, step_theta, step_z) = utils_planes_neighbors(nax1, nax2, nax3);
        let eps_step = 0.1;
        let aa = [0.0, 0.0, 0.0];
        let bb = [0., 0.0, step_z + eps_step];

        let CartesianCoordinates(aa) = CylindricalCoordinates(aa).into();
        let CartesianCoordinates(bb) = CylindricalCoordinates(bb).into();

        let cell1_id = mesh.cell_from_coordinates(&aa).unwrap();
        let cell2_id = mesh.cell_from_coordinates(&bb).unwrap();

        assert!(cell1_id != cell2_id);

        let (plane, axis) = mesh.get_interface_plane(cell1_id, cell2_id);
        assert_eq!(axis, 2);
        let m = mesh.cell_points(cell1_id);
        let mp = mesh.get_cell_edge(1, m[1]);
        let mn = mesh.get_cell_edge(1, m[1] + 1);
        let theta = aa[1];
        let theta_ext = theta - step_theta / 2.;
        let theta_in = theta + step_theta / 2.;

        // let mut origin: Vec<f64> = (0..3).map(|i| mesh.get_cell_center(i, m[i])).collect();
        // origin[2] = step_z;

        // assert!(
        //     origin == plane.origin.0,
        //     "{:?} {:?}",
        //     origin,
        //     plane.origin.0
        // );

        assert!(
            (plane.extent_v[0] - mp).abs() < 1e-8,
            "1 {} expected {}",
            plane.extent_v[0],
            theta_ext
        );
        assert!(
            (plane.extent_v[1] - mn).abs() < 1e-8,
            "2 {} expected {}",
            theta_in,
            0.
        );

        assert!(
            (plane.extent_u[0] - 0.0).abs() < 1e-8,
            "3 {} expected {}",
            plane.extent_u[0],
            0.
        );
        assert!(
            (plane.extent_u[1] - step_r).abs() < 1e-8,
            "4 {} expected {}",
            plane.extent_u[1],
            step_z
        );
    }

    #[test]
    fn t_neighbors_cylindrical() {
        use NeighborDirection::*;

        let mesh = ref_mesh_cyclindrical();

        let assert_neighbors =
            |a: CylindricalCoordinates, b: CylindricalCoordinates, expected: NeighborDirection| {
                let CartesianCoordinates(aa) = a.into();

                let CartesianCoordinates(bb) = b.into();

                let id1 = mesh
                    .cell_from_coordinates(&aa)
                    .expect("Test neighbors: coordinates for cell a are outside the mesh.");
                let id2 = mesh
                    .cell_from_coordinates(&bb)
                    .expect("Test neighbors: coordinates for cell b are outside the mesh.");

                let neighbors = mesh.are_cell_neighbor(id1, id2);
                assert!(
                    neighbors == expected,
                    "Assertion failed: expected {:?}, got {:?}",
                    expected,
                    neighbors
                );
            };

        // Fixed coordinates and offsets for testing
        let fix_i = 2.2;
        let offset_i = mesh.mesh_step_axis(0);
        let fix_j = 0.;
        let offset_j = 0.68;
        let fix_k = 4.0;
        let offset_k = 0.9;

        // Assert neighbor relationships in different directions

        // Testing in X direction
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i + offset_i, fix_j, fix_k]),
            XPlus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i + offset_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            XMinus,
        );

        // Testing in Y direction
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j + offset_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            YMinus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j + offset_j, fix_k]),
            YPlus,
        );

        // Testing in Z direction
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k + offset_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            ZMinus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k + offset_k]),
            ZPlus,
        );

        // Testing non-neighbor cases
        assert_neighbors(
            CylindricalCoordinates([0., fix_j, fix_k + offset_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            NotNeighbors,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([0., fix_j, fix_k + offset_k]),
            NotNeighbors,
        );
        let little_offset = mesh.mesh_step_axis(0) * 1.005;
        let c1: f64 = mesh.get_cell_edge(0, 2);
        assert_neighbors(
            CylindricalCoordinates([c1, fix_j, fix_k]),
            CylindricalCoordinates([c1 + little_offset, fix_j, fix_k]),
            NotNeighbors,
        );
    }
}
