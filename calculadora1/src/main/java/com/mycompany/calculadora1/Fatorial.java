/*
 * Click nbfs://nbhost/SystemFileSystem/Templates/Licenses/license-default.txt to change this license
 * Click nbfs://nbhost/SystemFileSystem/Templates/Classes/Class.java to edit this template
 */
package com.mycompany.calculadora1;

/**
 *
 * @author guilh
 */
public class Fatorial {
    private int num = 0;
    private String texto;
    private int fatorial;
    
    public void calcFatorial(int n){
        num = n;
        int f = 1;
        String s = "";
        for (int c = n; c > 1; c--) {
            f *= c;
            s += c + " x ";      
        }
        s += "1 = ";
        fatorial = f;
        texto = s;
    }  
    
    
   public int getFatorial(){
       return this.fatorial;
   }
   
   public String getFormula(){
       return this.texto;
   }
}
