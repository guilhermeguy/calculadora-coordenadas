/*
 * Click nbfs://nbhost/SystemFileSystem/Templates/Licenses/license-default.txt to change this license
 * Click nbfs://nbhost/SystemFileSystem/Templates/Classes/Class.java to edit this template
 */
/********************************************************************
A seguir estão as funções escritas em PHP que realizam conversão de 
coordenadas de projeção para coordenadas geográficas e vice versa  e
que são utilizadas na Calculadora Geografica do INPE
(www.dpi.inpe.br/calcula/)

Referencia:
SNYDER, John Parr. Map projections--A working manual. 
US Government Printing Office, 1987.

********************************************************************/
package com.mycompany.calculadora1;

/**
 *
 * @author guilh
 */
public class calcCoordenadas {
    private static final double PI = Math.PI;
    private int offy = 10000000;
    private double semiEixo = 6378137.0;
    // private double achat = 1.0 /298.257223563;
    private double achat = 0.003352811;
    private double coordX = 0;
    private double coordY = 0;

    /********************************************************************
     geo_to_utm
     Autor        : Julio Cesar Lima d'Alge                   jul-88
     Conversao p/ Php : Luis Maurano                         2008
     Resumo        : transforma coordenadas geodesicas em coordenadas UTM
     Entradas    : 
                 lat        coordenadas geodesicas (em radianos)
                 lon        coordenadas geodesicas (em radianos)
                 lon_mc     meridiano central (em radianos)
                 semi_eixo  semi_eixo_maior do elipsoide
                 achat      achatamento do elipsoide
                 hemis        norte ou sul
     Saidas        : x y coordenadas UTM (em metros)
    ********************************************************************/
    public void geoToUtm(double lat, double lon,
            String hemis, double lonMc){
        if ("norte".equalsIgnoreCase(hemis)) {
            this.offy = 0;
        } else {
            this.offy = 10000000;
        }
        lonMc = Math.toRadians(lonMc);
        double k0 = 0.9996;
        double equad = 2 * achat - Math.pow(achat, 2);
        double elinquad = equad / (1 - equad);
        
        lat = Math.toRadians(lat);
        lon = Math.toRadians(lon);
        double aux1 = Math.pow(equad, 2);
        double aux2 = aux1 * equad;
        double aux3 = Math.sin(2 * lat);
        double aux4 = Math.sin(4 * lat);
        double aux5 = Math.sin(6 * lat);
        double aux6 = (1 - equad / 4 - 3 * aux1 / 64 - 5 * aux2 / 256) * lat;
        double aux7 = (3 * equad / 4 - 3 * aux1 / 32 + 45 * aux2 / 1024) * aux3;
        double aux8 = (15 * aux1 / 256 + 45 * aux2 / 1024) * aux4;
        double aux9 = (35 * aux2 / 3072) * aux5;

        double n = semiEixo / Math.sqrt(1 - equad * Math.pow(Math.sin(lat), 2));
        double t = Math.pow(Math.tan(lat), 2);
        double c = elinquad * Math.pow(Math.cos(lat), 2);
        double ag = (lon - lonMc) * Math.cos(lat);
        double m = semiEixo * (aux6 - aux7 + aux8 - aux9);

        double aux10 = (1 - t + c) * Math.pow(ag, 3) / 6;
        double aux11 = (5 - 18 * t + Math.pow(t, 2) + 72 * c - 58 * elinquad) * Math.pow(ag, 5) / 120;
        double aux12 = (5 - t + 9 * c + 4 * Math.pow(c, 2)) * Math.pow(ag, 4) / 24;
        double aux13 = (61 - 58 * t + Math.pow(t, 2) + 600 * c - 330 * elinquad) * Math.pow(ag, 6) / 720;

        this.coordX = 500000 + k0 * n * (ag + aux10 + aux11);
        this.coordY = this.offy + k0 * (m + n * Math.tan(lat) * (Math.pow(ag, 2) / 2 + aux12 + aux13));
    }

    public double getCoordX() {
        return coordX;
    }

    public double getCoordY() {
        return coordY;
    }
}