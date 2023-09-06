# eterm-parser
a parser library in rust for eterm command that eterm server response text.

# Example: parse av text
```rust
let text = r" 03AUG(THU) PKXSHA VIA KN  
1- *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
>   MU2104      GQ UQ ZQ                                            -- T3 02:20
    MU2159  DS# J7 C5 D4 Q2 IQ YA BA MA EA HQ     SHA 1400   1620   32L 0^S  E  
>               KA LA NQ RQ SQ VQ TQ GQ ZQ                          T3 T2 07:20
2  *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
>   MU2104      GQ UQ ZQ                                            -- T3 02:20
   *MU3502  DS# YA BS MA ES KA LS NA RA SQ VQ     PVG 1500   1720   32S 0^S  E  
>   HO1212                                                          T3 T2 08:20
3   KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
>               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
    FM9530  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
>               KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35
4+  KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
>               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
   *MU8610  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
>   FM9530      KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35";
if let Ok(info) = eterm_parser::parse_av(text){
    assert_eq!(info.dpt, Some("PKX".to_owned()));
    assert_eq!(info.arr, Some("SHA".to_owned()));
    assert_eq!(info.date, Some("03AUG".to_owned()));
} else {
    assert_eq!(true, false);
}
```
# Example: parse fd text
```rust
let text=r"FD:KMGCTU/05SEP23/KY                   /CNY /TPM   744/                         
01 KY/J     / 5100.00=10200.00/J /C/  /   .   /25DEC19        /J000  PFN:01    
02 KY/G     / 1700.00= 3400.00/G /Y/  /   .   /25DEC19        /J000  PFN:02    
03 KY/Y     / 1700.00= 3400.00/Y /Y/  /   .   /25DEC19        /J000  PFN:03    
04 KY/B     / 1680.00= 3360.00/B /Y/  /   .   /25DEC19        /J000  PFN:04    
05 KY/M     / 1580.00= 3160.00/M /Y/  /   .   /25DEC19        /J000  PFN:05    
06 KY/M1    / 1500.00= 3000.00/M /Y/  /   .   /25DEC19        /J000  PFN:06    
07 KY/U     / 1410.00= 2820.00/U /Y/  /   .   /25DEC19        /J000  PFN:07    
08 KY/H     / 1330.00= 2660.00/H /Y/  /   .   /25DEC19        /J000  PFN:08    
09 KY/Q     / 1240.00= 2480.00/Q /Y/  /   .   /25DEC19        /J000  PFN:09    
10 KY/Q1    / 1160.00= 2320.00/Q /Y/  /   .   /25DEC19        /J000  PFN:10    
11 KY/V     / 1070.00= 2140.00/V /Y/  /   .   /25DEC19        /J000  PFN:11    
12 KY/V1    /  990.00= 1980.00/V /Y/  /   .   /25DEC19        /J000  PFN:12    
13 KY/W     /  900.00= 1800.00/W /Y/  /   .   /25DEC19        /J000  PFN:13    
14 KY/S     /  820.00= 1640.00/S /Y/  /   .   /25DEC19        /J000  PFN:14    
15 KY/E     /  730.00= 1460.00/E /Y/  /   .   /25DEC19        /J000  PFN:15    
                                                                                
PAGE 1/1       /LPRIC/C52DZF3YARTGI11                                           ";
if let Ok(info) = eterm_parser::parse_fd(text){
    assert_eq!(info.org, Some("KMG".to_owned()));
} else {
    assert_eq!(true, false);
}
```