/*
 * Click nbfs://nbhost/SystemFileSystem/Templates/Licenses/license-default.txt to change this license
 * Click nbfs://nbhost/SystemFileSystem/Templates/Classes/Class.java to edit this template
 */
package com.mycompany.calculadora1;

/**
 *
 * @author guilherme
 */
public class TestCalcCoordenadas {
    public static void main(String[] args) {
        calcCoordenadas calc = new calcCoordenadas();

        // Aachen, Germany
        double latAachen = 50.7753;
        double lonAachen = 6.0839;
        double lonMcAachen = 6.0;
        calc.geoToUtm(latAachen, lonAachen, "norte", lonMcAachen);
        System.out.println("Aachen - Coord X: " + calc.getCoordX() + ", Coord Y: " + calc.getCoordY());

        // Belo Horizonte, Brazil
        double latBeloHorizonte = -19.9245;
        double lonBeloHorizonte = -43.9352;
        double lonMcBeloHorizonte = -45.000;
        calc.geoToUtm(latBeloHorizonte, lonBeloHorizonte, "sul", lonMcBeloHorizonte);
        System.out.println("Belo Horizonte - Coord X: " + calc.getCoordX() + ", Coord Y: " + calc.getCoordY());
    }
}
