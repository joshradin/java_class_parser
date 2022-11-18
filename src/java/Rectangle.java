package com.example;

public class Rectangle {

    private double width;
    private double length;

    public Rectangle(double width, double length) {
        this.width = width;
        this.length = length;
    }



    public double getArea() {
        return this.width * this.length;
    }
}