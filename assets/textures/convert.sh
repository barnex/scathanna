#! /bin/bash

cd hi
for f in *.png; do
	echo $f;
	convert -scale 12.5% $f ../lo/$f;
done;


cd ../hi
for f in *.png; do
	echo $f;
	convert $f $(echo $f | sed 's/png/jpg/g');
done;

cd ../lo
for f in *.png; do
	echo $f;
	convert $f $(echo $f | sed 's/png/jpg/g');
done;