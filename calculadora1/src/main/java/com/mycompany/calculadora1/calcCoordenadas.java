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
    private double pi = Math.PI;
    private int offy = 10000000;
    private double coordX = 0;
    private double coordY = 0;
    
    
  /********************************************************************
 geo_to_utm				       
 Autor		: Julio Cesar Lima d'Alge		       jul-88
 Conversao p/ Php	: Luis Maurano						 2008
 Resumo		: transforma coordenadas geodesicas em coordenadas UTM
 Entradas	: 
 			lat		coordenadas geodesicas (em radianos)
		  lon		coordenadas geodesicas (em radianos)
		  lon_mc	meridiano central (em radianos)
		  semi_eixo	semi_eixo_maior do elipsoide
		  achat		achatamento do elipsoide
			hemis		norte ou sul
 Saidas		: x y coordenadas UTM (em metros)
********************************************************************/
    public void geo_to_utm(float lat, float lon,
            float semi_eixo,
            float achat,
            String hemis, float lon_mc){
        if("norte".equals(hemis)){
            this.offy = 0;
        }
        float k0 = 1 - (1/2500);
        double equad = (2 * achat - Math.pow(achat, 2));
        double elinquad = equad / (1 - equad);
        
        double aux1 = Math.pow(equad, 2);
        double aux2 = aux1 * equad;
        double aux3 = Math.sin(2 * lat);
        double aux4 = Math.sin(4 * lat);
        double aux5 = Math.sin(6 * lat);
        double aux6 = (1 - equad / 4 - 3 * aux1 / 64 - 5 * aux2 / 256) * lat;
        double aux7 = (3 * equad / 4 - 3 * aux1 / 32 + 45 * aux2 / 1024) * aux3;
        double aux8 = (15 * aux1 / 256 + 45 * aux2 / 1024) * aux4;
        double aux9 = (35 * aux2 / 3072) * aux5;
        
        double n = semi_eixo / Math.sqrt(1 - equad * Math.pow(Math.sin(lat), 2));
        double t = Math.pow(Math.tan(lat), 2);
        double c = elinquad * Math.pow(Math.cos(lat), 2);
        double ag = (lon - lon_mc) * Math.cos(lat);
        double m = semi_eixo * (aux6 - aux7 + aux8 - aux9);
        
        double aux10 = (1 - t + c) * ag * ag * ag / 6;
        double aux11 = (5 - 18 * t + t * t + 72 * c - 58 * elinquad) * (Math.pow(ag, 5)) / 120;
        double aux12 = (5 - t + 9 * c + 4 * c * c) * ag * ag * ag * ag / 24;
        double aux13 = (61 - 58 * t + t * t + 600 * c - 330 * elinquad) * (Math.pow(ag, 6)) / 720;
        
        this.coordX = 500000 + k0 * n * (ag + aux10 + aux11);
        this.coordY = this.offy + k0 * (m + n * Math.tan(lat) * (ag * ag / 2 + aux12 + aux13));
        
    }

    public double getCoordX() {
        return coordX;
    }

    public double getCoordY() {
        return coordY;
    }
    
    
}

