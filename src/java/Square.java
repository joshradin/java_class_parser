package com.example;

public class Square extends Rectangle implements Comparable<Square> {


    public Square(double length) {
        super(length, length);
    }


    @Override
    public int compareTo(Square other) {
        return this.getArea() - other.getArea() > 0.0 ? 1 : -1;
    }

    @Deprecated
    public boolean isSquare() {
        return true;
    }

}