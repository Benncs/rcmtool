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

from .pycmtool import *

__doc__ = []
__doc__.extend([pycmtool.data.__doc__])

__all__ = []
__doc__ = pycmtool.__doc__
if hasattr(pycmtool, "__all__"):
    __all__ = pycmtool.__all__


if hasattr(pycmtool, "__all__"):
    __all__ = pycmtool.__all__


__all__.extend([])
