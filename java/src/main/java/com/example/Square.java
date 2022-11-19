package com.example;

public class Square extends Rectangle implements Comparable<Rectangle>{

    public Square(double side) {
        super(side, side);
    }

    @Override
    public int compareTo(Rectangle other) {
        return (int) (this.getArea() - other.getArea());
    }
}