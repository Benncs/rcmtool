use std::cell;
use std::fmt::write;

use super::CompartmentMesh;
use crate::CoreError;
use crate::coordinates::{CartesianCoordinates, CylindricalCoordinates};
use vtkio::Vtk;
use vtkio::model::{UnstructuredGridPiece, VertexNumbers};

trait VtkCmReader<T: CompartmentMesh> {}
trait VtkCmWriter {
    fn get_grid(&self) -> vtkio::model::UnstructuredGridPiece;

    fn get_cell_coordinates(&self) -> Vec<[f64; 24]>;
}

pub trait VtkCm {
    fn write(&self, path: impl AsRef<std::path::Path>) -> Result<(), CoreError>;
    fn read(path: impl AsRef<std::path::Path>) -> Result<Vtk, CoreError>;
}

impl VtkCmWriter for Box<dyn CompartmentMesh> {
    //Naive c++ adaptation, TODO: find way to improve method
    fn get_cell_coordinates(&self) -> Vec<[f64; 24]> {
        let n_cells = self.number_cell();
        let mut cell_coordinates = Vec::with_capacity(n_cells);

        for cell_id in 0..n_cells {
            let mut cell_array = [0.0; 24];

            let indices = self.cell_points(cell_id);

            let vertex_indices = [
                [0, 0, 0],
                [1, 0, 0],
                [1, 1, 0],
                [0, 1, 0],
                [0, 0, 1],
                [1, 0, 1],
                [1, 1, 1],
                [0, 1, 1],
            ];

            for (idx, vertex) in vertex_indices.iter().enumerate() {
                let x_index = indices[0] + vertex[0];
                let y_index = indices[1] + vertex[1];
                let z_index = indices[2] + vertex[2];

                let x = self.get_cell_edge(0, x_index);
                let y = self.get_cell_edge(1, y_index);
                let z = self.get_cell_edge(2, z_index);
                let CartesianCoordinates([x, y, z]) = CylindricalCoordinates([x, y, z]).into();

                let base_index = idx * 3;
                cell_array[base_index] = x;
                cell_array[base_index + 1] = y;
                cell_array[base_index + 2] = z;
            }

            cell_coordinates.push(cell_array);
        }

        cell_coordinates
    }

    fn get_grid(&self) -> vtkio::model::UnstructuredGridPiece {
        let n_cells = self.number_cell();

        let mut points_vec = Vec::with_capacity(n_cells * 24);
        let mut connectivity = Vec::with_capacity(n_cells);
        let mut offsets = Vec::with_capacity(n_cells);

        let cell_coordinates = self.get_cell_coordinates();

        // for (_cell_id, coordinates) in cell_coordinates.iter().enumerate() {
        for coordinates in cell_coordinates.iter() {
            let mut cell_vertex_indices = Vec::new();

            for vertex_idx in 0..8 {
                let base_index = vertex_idx * 3;
                let x = coordinates[base_index];
                let y = coordinates[base_index + 1];
                let z = coordinates[base_index + 2];

                let point_index = points_vec.len() / 3;
                points_vec.push(x);
                points_vec.push(y);
                points_vec.push(z);
                cell_vertex_indices.push(point_index as u64);
            }

            connectivity.extend(cell_vertex_indices);
            offsets.push((connectivity.len()) as u64);
        }

        let types = vec![vtkio::model::CellType::Hexahedron; self.number_cell()];
        let cell_verts: VertexNumbers = VertexNumbers::XML {
            connectivity,
            offsets,
        };
        let points = vtkio::model::IOBuffer::F64(points_vec);
        let cells = vtkio::model::Cells { types, cell_verts };

        let test_data_array = vtkio::model::DataArray::scalars("Random", 1);
        let rd = (0..self.number_cell()).collect();
        let test_data_array = test_data_array.with_vec(rd);

        let mut data = vtkio::model::Attributes::new();

        data.cell
            .push(vtkio::model::Attribute::DataArray(test_data_array));

        UnstructuredGridPiece {
            points,
            cells,
            data,
        }
    }
}

impl VtkCm for Box<dyn CompartmentMesh> {
    fn write(&self, path: impl AsRef<std::path::Path>) -> Result<(), CoreError> {
        let version = vtkio::model::Version::new((1, 0));
        let title = String::from("CompartmentMesh");

        let grid = self.get_grid();
        let pieces = vtkio::model::Piece::Inline(Box::new(grid));
        let data = vtkio::model::DataSet::UnstructuredGrid {
            meta: None,
            pieces: vec![pieces],
        };
        let byte_order = vtkio::model::ByteOrder::native();
        let file_path = path.as_ref().into();
        let vtk = Vtk {
            version,
            title,
            byte_order,
            data,
            file_path: Some(file_path),
        };

        let mut vtk_bytes = Vec::<u8>::new();
        vtk.write_xml(&mut vtk_bytes).unwrap();
        std::fs::write(path, vtk_bytes).unwrap();

        Ok(())
    }
    fn read(path: impl AsRef<std::path::Path>) -> Result<Vtk, CoreError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::*;
    const NAX1: usize = 10;
    const MAX_AX1: f64 = 4.;

    const NAX2: usize = 10;
    const MAX_AX2: f64 = 2.;

    const NAX3: usize = 10;
    const MAX_AX3: f64 = 10.;

    fn ref_mesh_cyclindrical() -> Box<dyn CompartmentMesh> {
        let ax1 = AxisDescriptor::new(0., MAX_AX1, NAX1);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, NAX2);
        let ax3 = AxisDescriptor::new(0., MAX_AX3, NAX3);
        get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3])
    }

    #[test]
    fn test() {
        let grid = ref_mesh_cyclindrical();
        grid.write("/tmp/test.vtu");
    }
}
