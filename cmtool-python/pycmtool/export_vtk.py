# SPDX-License-Identifier: GPL-3.0-or-later
"""
This file is part of Compartment Modelling Tool Project (CMT).

Compartment Modelling Tool Project (CMT) is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Compartment Modelling Tool Project (CMT) is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with Compartment Modelling Tool Project (CMT). If not, see <https://www.gnu.org/licenses/>.

Please contact:

- Casale Benjamin: casale@insa-toulouse.fr
"""

__all__ = []
import json
import os
from typing import Dict, List

import numpy as np
import vtk
import vtkmodules.util.numpy_support as npvtk

# from cmtool.wcma_read import RawDataFlux

# def mk_point(coordinates):
#   points = vtk.vtkPoints()
#   for coordinate in coordinates:
#     points.InsertNextPoint(coordinate)
#   return points


# def save_points(filename,coordinates):
#   points = mk_point(coordinates)
#   polydata = vtk.vtkPolyData()
#   polydata.SetPoints(points)

#   # Write the PolyData to a VTK XML file
#   writer = vtk.vtkXMLPolyDataWriter()
#   writer.SetFileTypeToBinary()
#   writer.SetFileName(f"{filename}.vtp")
#   writer.SetInputData(polydata)
#   writer.Update()
#   writer.Write()


# def write_vtk(filename,coordinates,*args):
#     points = mk_point(coordinates)
#     polydata = vtk.vtkPolyData()
#     polydata.SetPoints(points)

#     for i in args:
#       polydata.GetPointData().AddArray(i)

#     # Write the PolyData to a VTK XML file
#     writer = vtk.vtkXMLPolyDataWriter()
#     writer.SetFileName(filename)
#     writer.SetInputData(polydata)
#     writer.SetDataModeToBinary()
#     writer.Update()
#     writer.Write()


def mk_scalar(np_array_data: np.ndarray, scalar_name: str):
    """!
    @brief Make VTK scalar from numpy array.

    This function converts a numpy array into a VTK scalar array. It sets the number of components to 1
    and assigns the specified scalar name.

    @param np_array_data (numpy.ndarray): The input numpy array containing scalar data.
    @param scalar_name (str): The name to assign to the VTK scalar array.

    @return vtk.vtkFloatArray: The VTK scalar array containing the data.

    @example
    import numpy as np
    from cmtool.vtk import mk_scalar

    number_of_cell = ...
    scalar = np.zeros((number_of_cell,))
    mk_scalar(scalar,"my_scalar")
    """
    scalar_array = npvtk.numpy_to_vtk(np_array_data)
    scalar_array.SetNumberOfComponents(1)
    scalar_array.SetName(scalar_name)
    return scalar_array


def read_scalar(filename, name) -> np.ndarray:
    """!
    @brief Read scalar data from a VTK XML unstructured grid file.

    This function reads scalar data from a VTK XML unstructured grid file. It retrieves the scalar
    data associated with the specified name from the cell data of the grid.

    @param filename str: The path to the VTK XML unstructured grid file.
    @param name str: The name of the scalar data array to be retrieved.

    @return numpy.ndarray: The scalar data as a numpy array.

    @example
    scalar_data = read_scalar("example.vtu", "Pressure")
    """
    reader = vtk.vtkXMLUnstructuredGridReader()
    reader.SetFileName(filename)
    # Perform the read operation
    reader.Update()
    grid = reader.GetOutput()
    vtk_array = grid.GetCellData().GetArray(name)
    return npvtk.vtk_to_numpy(vtk_array)


def append_scalar(filename, destination, *args):
    """!
    @brief Append scalar arrays to an existing VTK XML unstructured grid file.

    This function reads an existing VTK XML unstructured grid file, appends the specified scalar arrays to the cell data,
    and writes the modified grid to a new VTK XML file.

    @param filename str: The path to the input VTK XML unstructured grid file.
    @param destination str: The path to the output VTK XML unstructured grid file.
    @param args vtk.vtkDataArray: Variable-length list of VTK scalar arrays to append to the cell data.

    @return None

    @example
    pressure_array = vtk.vtkFloatArray()
    pressure_array.SetName("Pressure")
    # ...populate pressure_array with data...
    append_scalar("input.vtu", "output.vtu", pressure_array)
    """
    # Read the input VTK XML unstructured grid file
    reader = vtk.vtkXMLUnstructuredGridReader()
    reader.SetFileName(filename)
    reader.Update()
    grid = reader.GetOutput()

    # Append scalar arrays to the cell data
    for scalar_array in args:
        grid.GetCellData().AddArray(scalar_array)

    # Write the modified grid to a new VTK XML file
    writer = vtk.vtkXMLUnstructuredGridWriter()
    writer.SetFileName(destination)
    writer.SetInputData(grid)
    writer.SetDataModeToBinary()
    writer.Update()
    writer.Write()


def write_json_series(
    destination: str, name: str, file_entries: List[Dict[str, str]]
) -> None:
    """!
    @brief Write a JSON file for a VTK file series.

    This function creates a JSON file that describes a series of VTK files. It includes
    the version of the file-series and a list of file entries provided by the user.

    @param destination str: The directory path where the JSON file will be saved.
    @param name str: The base name for the JSON file (without extension).
    @param file_entries list: A list of file entries (each entry is a dictionary) that specifies the files in the series.

    @return None

    @example
    file_entries = [
        {"name": "step0.vtu", "time": 0.0},
        {"name": "step1.vtu", "time": 1.0}
    ]
    write_json_series("/path/to/destination", "series_name", file_entries)
    """
    json_content = {"file-series-version": "1.0", "files": file_entries}

    # Write the JSON file
    json_file_path = f"{destination}/{name}.vtu.series"
    with open(json_file_path, "w") as json_file:
        json.dump(json_content, json_file, indent=4)


def mk_series(filepath: str, destination: str, series_name: str, t: np.ndarray, *args):
    """!
    @brief Generate a series of VTK files and a corresponding JSON series file.

    This function creates a series of VTK XML unstructured grid files (.vtu) and a JSON file that describes
    the series. For each time step provided in the `t` array, it generates a VTK file with appended scalar
    data arrays and saves these files in the specified destination directory. It also generates a JSON file
    to represent the file series.

    @param filepath str: The path to the input VTK XML unstructured grid file.
    @param destination str: The directory where the series files and JSON descriptor will be saved.
    @param series_name str: The base name for the series files and the JSON descriptor.
    @param t numpy.ndarray: An array of time steps corresponding to each VTK file in the series.
    @param args Variable-length argument list of tuples, where each tuple contains:
                - data numpy.ndarray: The scalar data array to be appended to each VTK file.
                - name str: The name of the scalar data array.

    @return None

    @example
    t = np.array([0.0, 1.0, 2.0])
    pressure_data = np.random.random((3, 100))  # Example scalar data with 3 time steps and 100 cells
    temperature_data = np.random.random((3, 100))  # Another example scalar data
    mk_series("input.vtu", "output_directory", "simulation_series", t, (pressure_data, "Pressure"), (temperature_data, "Temperature"))
    """
    file_entries = []
    n_t = len(t)
    root = f"{destination}/{series_name}"
    if not os.path.exists(root):
        os.makedirs(root)

    for i in range(n_t):
        vtp_result = f"{root}/{series_name}_{i}.vtu"
        scalar = []
        for s in args:
            data = s[0]
            name = s[1]
            scalar.append(mk_scalar(data[i], name))
        append_scalar(filepath, vtp_result, *scalar)
        file_entries.append({"name": f"{series_name}_{i}.vtu", "time": t[i]})
    write_json_series(root, series_name, file_entries)


__all__.extend(["mk_series", "read_scalar", "write_json_series", "mk_scalar"])
