
xml:
	tidy -iqm -wrap 0 -xml splits-direct.lss
	tidy -iqm -wrap 0 -xml layout-direct.lsl
	tidy -iqm -wrap 0 -xml splits-lswasr.lss
	tidy -iqm -wrap 0 -xml layout-lswasr.lsl
	tidy -iqm -wrap 0 -xml splits-control.lss
	tidy -iqm -wrap 0 -xml layout-control.lsl
