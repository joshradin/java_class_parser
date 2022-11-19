package com.example;

public class Square extends Rectangle implements Comparable<Square>{

    public Square(double side) {
        super(side, side);
    }

    @Override
    public int compareTo(Square other) {
        return (int) (this.getArea() - other.getArea());
    }
}