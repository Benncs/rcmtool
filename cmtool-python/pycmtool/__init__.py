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

from typing import Tuple, Optional

import scipy.sparse

from . import pycmtool as _native
from .pycmtool import *  # noqa: F403

__doc__ = _native.__doc__

__all__ = list(getattr(_native, "__all__", []))
__all__.extend(["sparse_array_from_triplet", "get_sparse_transition_matrix"])


def get_sparse_transition_matrix(it):
    return sparse_array_from_triplet(*it.transition)


def sparse_array_from_triplet(
    row_indices: Tuple[int, ...],
    col_indices: Tuple[int, ...],
    values: Tuple[float, ...],
    shape: Optional[Tuple[int, int]] = None,
) -> scipy.sparse.coo_array:
    """
    Construct a sparse COO array from row indices, column indices, and values.

    Args:
        row_indices: Array of row indices.
        col_indices: Array of column indices.
        values: Array of values.
        shape: Optional shape of the resulting sparse array. If None, inferred from indices.

    Returns:
        A scipy.sparse.coo_array constructed from the triplets.
    """
    if shape is None:
        max_row = max(row_indices) if row_indices else 0
        max_col = max(col_indices) if col_indices else 0
        shape = (max_row + 1, max_col + 1)

    sparse_array = scipy.sparse.coo_array(
        (values, (row_indices, col_indices)),
        shape=shape,
        copy=False,
    )
    return sparse_array
