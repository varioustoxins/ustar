data_protein_structure

_entry.id                      1ABC
_entry.title                   'Test protein structure with errors'
_entry.method                  'X-RAY DIFFRACTION'
_entry.resolution              2.1
_entry.space_group             'P 21 21 21'

loop_
_atom_site.group_PDB
_atom_site.id
_atom_site.type_symbol
_atom_site.label_atom_id
_atom_site.label_comp_id
_atom_site.label_seq_id
_atom_site.Cartn_x
_atom_site.Cartn_y
_atom_site.Cartn_z
_atom_site.occupancy
_atom_site.B_iso_or_equiv
ATOM 1  N  N   MET 1   20.154  6.718   22.970  1.00  25.45
ATOM 2  CA CA  MET 1   21.618  6.696   23.156  1.00  24.32
ATOM 3  C  C   MET 1   22.061  5.721   24.236  1.00  23.89
ATOM 4  O  O   MET 1   21.265  4.956   24.796  1.00  24.56

# This is where the error occurs - unclosed string
_experimental.description      "This is an unclosed string that will cause parsing errors
_experimental.temperature      298.0
_experimental.pH               7.4
_experimental.buffer           'TRIS-HCl 20mM, NaCl 150mM'

loop_
_pdbx_database_status.status_code
_pdbx_database_status.entry_id
_pdbx_database_status.deposit_date
_pdbx_database_status.process_site
REL  1ABC  1995-02-15  RCSB
ADIT 1ABC  1994-11-23  RCSB

_struct.entry_id               1ABC
_struct.title                  'Example structure'
_struct.pdbx_descriptor        'TRANSFERASE'
_struct.pdbx_model_type_details ?

_cell.length_a                 54.321
_cell.length_b                 58.234
_cell.length_c                 61.789
_cell.angle_alpha              90.0
_cell.angle_beta               90.0
_cell.angle_gamma              90.0