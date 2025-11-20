data_nmr_experiment

_nmr_spectrometer.manufacturer         'Bruker'
_nmr_spectrometer.model                'Avance III HD'
_nmr_spectrometer.field_strength       800.0
_nmr_spectrometer.probe_type           'TCI CryoProbe'

_sample.label                          'Protein Sample 1'
_sample.type                           'solution'
_sample.solvent_system                 '90% H2O/10% D2O'
_sample.pH                             6.8
_sample.temperature                    298
_sample.ionic_strength                 0.1
_sample.pressure                       'ambient'

loop_
_experiment.name
_experiment.type
_experiment.pulse_sequence
_experiment.sample_volume
_experiment.acquisition_time
'1D 1H'        '1D'        'zg30'      600.0  2.5
'2D HSQC'      '2D'        'hsqcetgp'  600.0  8.2
'2D NOESY'     '2D'        'noesyph'   600.0  12.1

# This is where the error occurs - tag used instead of value
_chemical_shift_reference.indirect_shift_ratio  _chemical_shift_reference.mol_common_name
_chemical_shift_reference.atom_type             H
_chemical_shift_reference.atom_isotope_number   1
_chemical_shift_reference.atom_group            'CH3'
_chemical_shift_reference.concentration_value   1.0
_chemical_shift_reference.concentration_units   mM
_chemical_shift_reference.geometry              tetrahedral

_software.classification               'collection'
_software.name                         'TopSpin'
_software.version                      '4.0.7'
_software.vendor                       'Bruker BioSpin'

loop_
_assigned_chemical_shift.assembly_atom_ID
_assigned_chemical_shift.entity_assembly_ID
_assigned_chemical_shift.entity_ID
_assigned_chemical_shift.comp_index_ID
_assigned_chemical_shift.seq_ID
_assigned_chemical_shift.comp_ID
_assigned_chemical_shift.atom_ID
_assigned_chemical_shift.atom_type
_assigned_chemical_shift.atom_isotope_number
_assigned_chemical_shift.val
_assigned_chemical_shift.val_err
1  1  1  1  1  MET  H    H  1  8.234  0.02
2  1  1  1  1  MET  HA   H  1  4.521  0.01
3  1  1  1  1  MET  HB2  H  1  2.134  0.01