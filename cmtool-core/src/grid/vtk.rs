use std::cell;
use std::fmt::write;

use super::CompartmentMesh;
use crate::CoreError;
use crate::coordinates::CartesianCoordinates;
use vtkio::Vtk;
use vtkio::model::{UnstructuredGridPiece, VertexNumbers};

trait VtkCmReader<T: CompartmentMesh> {}
trait VtkCmWriter {
    fn set_cells(&self) -> Result<(), CoreError>;
    fn get_grid(&self) -> vtkio::model::UnstructuredGridPiece;
}

pub trait VtkCm {
    fn write(&self, path: impl AsRef<std::path::Path>) -> Result<(), CoreError>;
    fn read(path: impl AsRef<std::path::Path>) -> Result<Vtk, CoreError>;
}

impl VtkCmWriter for Box<dyn CompartmentMesh> {
    fn set_cells(&self) -> Result<(), CoreError> {
        const N_VERTICES: usize = 8; //Cells are cube
        let mut cell_coordinates =
            vec![[CartesianCoordinates::default(); N_VERTICES]; self.number_cell()];

        for i_cell in 0..cell_coordinates.len() {}

        todo!()
    }

    fn get_grid(&self) -> vtkio::model::UnstructuredGridPiece {
        let types = vec![vtkio::model::CellType::Hexahedron; self.number_cell()];
        let types_len = types.len();

        let connectivity: Vec<u64> = (0..types_len * 8).map(|e| (e % 8) as u64).collect();

        let offsets: Vec<u64> = (1..=types_len).map(|e| (e * 8) as u64).collect();

        let cell_verts: VertexNumbers = VertexNumbers::XML {
            connectivity,
            offsets,
        };

        let points = vtkio::model::IOBuffer::F64(vec![0.; self.number_cell()]);
        let cells = vtkio::model::Cells { types, cell_verts };

        let data = vtkio::model::Attributes::default();
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
