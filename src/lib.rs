//! eterm parser
//! ==============
//!
//! This library contains text parser of the eterm common
//! command such as av,detr,fd,ml,pat,rt,etc.
//! 
//! impl zero allocation and zero cost with lifetime and &str.
//!
//! [Docs](https://docs.rs/eterm-parser/) |
//! [Github](https://github.com/bmrxntfj/eterm-parser/) |
//! [Crate](https://crates.io/crates/eterm-parser)
//!
//!
//! Example: parse av text
//! -------------------------------
//!
//! ```
//! let text = r" 03AUG(THU) PKXSHA VIA KN  
//! 1- *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
//! >   MU2104      GQ UQ ZQ                                            -- T3 02:20
//!     MU2159  DS# J7 C5 D4 Q2 IQ YA BA MA EA HQ     SHA 1400   1620   32L 0^S  E  
//! >               KA LA NQ RQ SQ VQ TQ GQ ZQ                          T3 T2 07:20
//! 2  *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
//! >   MU2104      GQ UQ ZQ                                            -- T3 02:20
//!    *MU3502  DS# YA BS MA ES KA LS NA RA SQ VQ     PVG 1500   1720   32S 0^S  E  
//! >   HO1212                                                          T3 T2 08:20
//! 3   KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
//! >               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
//!     FM9530  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
//! >               KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35
//! 4+  KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
//! >               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
//!    *MU8610  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
//! >   FM9530      KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35";
//! if let Ok(info) = eterm_parser::parse_av(text){
//!     assert_eq!(info.dpt, Some("PKX"));
//!     assert_eq!(info.arr, Some("SHA"));
//!     assert_eq!(info.date, Some("03AUG"));
//! } else {
//!     assert_eq!(true, false);
//! }
//! ```
//!
//! Example: parse fd text
//! -------------------------------
//!
//! ```
//! let text=r"FD:KMGCTU/05SEP23/KY                   /CNY /TPM   744/                         
//! 01 KY/J     / 5100.00=10200.00/J /C/  /   .   /25DEC19        /J000  PFN:01    
//! 02 KY/G     / 1700.00= 3400.00/G /Y/  /   .   /25DEC19        /J000  PFN:02    
//! 03 KY/Y     / 1700.00= 3400.00/Y /Y/  /   .   /25DEC19        /J000  PFN:03    
//! 04 KY/B     / 1680.00= 3360.00/B /Y/  /   .   /25DEC19        /J000  PFN:04    
//! 05 KY/M     / 1580.00= 3160.00/M /Y/  /   .   /25DEC19        /J000  PFN:05    
//! 06 KY/M1    / 1500.00= 3000.00/M /Y/  /   .   /25DEC19        /J000  PFN:06    
//! 07 KY/U     / 1410.00= 2820.00/U /Y/  /   .   /25DEC19        /J000  PFN:07    
//! 08 KY/H     / 1330.00= 2660.00/H /Y/  /   .   /25DEC19        /J000  PFN:08    
//! 09 KY/Q     / 1240.00= 2480.00/Q /Y/  /   .   /25DEC19        /J000  PFN:09    
//! 10 KY/Q1    / 1160.00= 2320.00/Q /Y/  /   .   /25DEC19        /J000  PFN:10    
//! 11 KY/V     / 1070.00= 2140.00/V /Y/  /   .   /25DEC19        /J000  PFN:11    
//! 12 KY/V1    /  990.00= 1980.00/V /Y/  /   .   /25DEC19        /J000  PFN:12    
//! 13 KY/W     /  900.00= 1800.00/W /Y/  /   .   /25DEC19        /J000  PFN:13    
//! 14 KY/S     /  820.00= 1640.00/S /Y/  /   .   /25DEC19        /J000  PFN:14    
//! 15 KY/E     /  730.00= 1460.00/E /Y/  /   .   /25DEC19        /J000  PFN:15    
//!                                                                                 
//! PAGE 1/1       /LPRIC/C52DZF3YARTGI11                                           ";
//! if let Ok(info) = eterm_parser::parse_fd(text){
//!     assert_eq!(info.org, Some("KMG"));
//! } else {
//!     assert_eq!(true, false);
//! }
//! ```
//!
//!

mod util;
/// The module include text parser and result type of response of av command.
pub mod av;
/// The module include text parser and result type of response of detr command.
pub mod detr;
/// The module include text parser and result type of response of fd command.
pub mod fd;
/// The module include text parser and result type of response of ml command.
pub mod ml;
/// The module include text parser and result type of response of pat command.
pub mod pat;
/// The module include text parser and result type of response of rt command.
pub mod pnr;

/// Parse av text that eterm server response.
///
/// # Examples
///
/// ```
/// let text = r" 03AUG(THU) PKXSHA VIA KN  
/// 1- *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
/// >   MU2104      GQ UQ ZQ                                            -- T3 02:20
///     MU2159  DS# J7 C5 D4 Q2 IQ YA BA MA EA HQ     SHA 1400   1620   32L 0^S  E  
/// >               KA LA NQ RQ SQ VQ TQ GQ ZQ                          T3 T2 07:20
/// 2  *KN6856  DS# JA C8 YA BA HA KA LA RQ SQ TQ  PKXXIY 0900   1120   321 0^B  E  
/// >   MU2104      GQ UQ ZQ                                            -- T3 02:20
///    *MU3502  DS# YA BS MA ES KA LS NA RA SQ VQ     PVG 1500   1720   32S 0^S  E  
/// >   HO1212                                                          T3 T2 08:20
/// 3   KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
/// >               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
///     FM9530  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
/// >               KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35
/// 4+  KN5730  DS# WA YA BA MA EA HA KA LA NA R6  PKXWNZ 0915   1145   73U 0^   E  
/// >               SQ VQ DQ TQ IQ ZQ U5 PQ GQ QS AQ                    -- T2 02:30
///    *MU8610  DS# J7 C7 D7 Q6 I4 YA BA MA EA HA     PVG 1545   1650   73E 0^   E  
/// >   FM9530      KA LA NA RA SA VA TA GS ZA                          T2 T1 07:35";
/// if let Ok(info) = eterm_parser::parse_av(text){
///     assert_eq!(info.dpt, Some("PKX"));
///     assert_eq!(info.arr, Some("SHA"));
///     assert_eq!(info.date, Some("03AUG"));
/// } else {
///     assert_eq!(true, false);
/// }
/// ```
pub fn parse_av(text: &str) -> anyhow::Result<av::Av> {
    av::Av::parse(text)
}

/// Parse detr text that eterm server response.
///
/// # Examples
///
/// ```
/// let text = r"ET PROCESSING IN PROGRESS   
/// AATK:TN/9992303753785   
/// ISSUED BY: AIR CHINA                 ORG/DST: HET/SIA                 ARL-D 
/// TOUR CODE: ZCC4000LC
/// PASSENGER: dwfei
/// EXCH:                               CONJ TKT:   
/// O FM:1HET CA    8113  S 21MAY 0815 OK S                        20K OPEN FOR USE 
///      --T2 RL:NZJ0JY  /  
///   TO: XIY   b
/// FC: M  21MAY23HET CA XIY308.00CNY308.00END  
/// FARE:           CNY  308.00|FOP:CC VI184
/// TAX:               EXEMPTCN|OI: 
/// TAX:            CNY 60.00YQ|                                                   +
/// ";
/// if let Ok(info) = eterm_parser::parse_detr(text){
///     assert_eq!(info.org, Some("HET"));
///     assert_eq!(info.dst, Some("SIA"));
/// } else {
///     assert_eq!(true, false);
/// }
/// ```
pub fn parse_detr(text: &str) -> anyhow::Result<detr::Detr> {
    detr::Detr::parse(text)
}

/// Parse pnr text that eterm server response.
///
/// # Examples
///
/// ```
/// let text = r"  **ELECTRONIC TICKET PNR**                                                     
///  1.石风芸CHD KE9SWE                                                             
///  2.  JD5324 Y   WE06SEP  DXJPKX RR1   1045 1310          E                      
///  3.KMG/T KMG/T 037968926796/KUNMING WKN TANG TRADING CO. LTD./ZHANGSAN      
///  4.T                                                                            
///  5.SSR FOID JD HK1 NI433101202105250023/P1                                      
///  6.SSR ADTK 1E BY KMG28AUG23/1742 OR CXL JD5324 Y06SEP                          
///  7.SSR TKNE JD HK1 DXJPKX 5324 Y06SEP 8989198306578/1/P1                        
///  8.SSR CHLD JD HK1 25MAY21/P1                                                   
///  9.OSI JD CTCT13320512490                                                       
/// 10.OSI JD CTCM15718791505/P1                                                    
/// 11.OSI JD ADT/8989198306575    ";
/// if let Ok(info) = eterm_parser::parse_pnr(text){
///     assert_eq!(info.pnr_code, Some("KE9SWE"));
/// } else {
///     assert_eq!(true, false);
/// }
/// ```
pub fn parse_pnr(text: &str) -> anyhow::Result<pnr::Pnr> {
    pnr::Pnr::parse(text)
}

/// Parse ml text that eterm server response.
///
/// # Examples
///
/// ```
/// let text = r"  **ELECTRONIC TICKET PNR**                                                     
/// 1.石风芸CHD KE9SWE                                                             
/// 2.  JD5324 Y   WE06SEP  DXJPKX RR1   1045 1310          E                      
/// 3.KMG/T KMG/T 037968926796/KUNMING WKN TANG TRADING CO. LTD./ZHANGSAN      
/// 4.T                                                                            
/// 5.SSR FOID JD HK1 NI433101202105250023/P1                                      
/// 6.SSR ADTK 1E BY KMG28AUG23/1742 OR CXL JD5324 Y06SEP                          
/// 7.SSR TKNE JD HK1 DXJPKX 5324 Y06SEP 8989198306578/1/P1                        
/// 8.SSR CHLD JD HK1 25MAY21/P1                                                   
/// 9.OSI JD CTCT13320512490                                                       
/// 10.OSI JD CTCM15718791505/P1                                                    
/// 11.OSI JD ADT/8989198306575    ";
/// if let Ok(info) = eterm_parser::parse_ml(text){
///     //assert_eq!(info.pnr_code, Some("KE9SWE"));
/// } else {
///     //assert_eq!(true, false);
/// }
/// ```
pub fn parse_ml(text: &str) -> anyhow::Result<ml::Ml> {
    ml::Ml::parse(text)
}

/// Parse fd text that eterm server response.
///
/// # Examples
///
/// ```
/// let text=r"FD:KMGCTU/05SEP23/KY                   /CNY /TPM   744/                         
/// 01 KY/J     / 5100.00=10200.00/J /C/  /   .   /25DEC19        /J000  PFN:01    
/// 02 KY/G     / 1700.00= 3400.00/G /Y/  /   .   /25DEC19        /J000  PFN:02    
/// 03 KY/Y     / 1700.00= 3400.00/Y /Y/  /   .   /25DEC19        /J000  PFN:03    
/// 04 KY/B     / 1680.00= 3360.00/B /Y/  /   .   /25DEC19        /J000  PFN:04    
/// 05 KY/M     / 1580.00= 3160.00/M /Y/  /   .   /25DEC19        /J000  PFN:05    
/// 06 KY/M1    / 1500.00= 3000.00/M /Y/  /   .   /25DEC19        /J000  PFN:06    
/// 07 KY/U     / 1410.00= 2820.00/U /Y/  /   .   /25DEC19        /J000  PFN:07    
/// 08 KY/H     / 1330.00= 2660.00/H /Y/  /   .   /25DEC19        /J000  PFN:08    
/// 09 KY/Q     / 1240.00= 2480.00/Q /Y/  /   .   /25DEC19        /J000  PFN:09    
/// 10 KY/Q1    / 1160.00= 2320.00/Q /Y/  /   .   /25DEC19        /J000  PFN:10    
/// 11 KY/V     / 1070.00= 2140.00/V /Y/  /   .   /25DEC19        /J000  PFN:11    
/// 12 KY/V1    /  990.00= 1980.00/V /Y/  /   .   /25DEC19        /J000  PFN:12    
/// 13 KY/W     /  900.00= 1800.00/W /Y/  /   .   /25DEC19        /J000  PFN:13    
/// 14 KY/S     /  820.00= 1640.00/S /Y/  /   .   /25DEC19        /J000  PFN:14    
/// 15 KY/E     /  730.00= 1460.00/E /Y/  /   .   /25DEC19        /J000  PFN:15    
///                                                                                 
/// PAGE 1/1       /LPRIC/C52DZF3YARTGI11                                           ";
/// if let Ok(info) = eterm_parser::parse_fd(text){
///     assert_eq!(info.org, Some("KMG"));
/// } else {
///     assert_eq!(true, false);
/// }
/// ```
pub fn parse_fd(text: &str) -> anyhow::Result<fd::Fd> {
    fd::Fd::parse(text)
}

/// Parse pat text that eterm server response.
///
/// # Examples
///
/// ```
/// let text = r"  **ELECTRONIC TICKET PNR**                                                     
/// 1.石风芸CHD KE9SWE                                                             
/// 2.  JD5324 Y   WE06SEP  DXJPKX RR1   1045 1310          E                      
/// 3.KMG/T KMG/T 037968926796/KUNMING WKN TANG TRADING CO. LTD./ZHANGSAN      
/// 4.T                                                                            
/// 5.SSR FOID JD HK1 NI433101202105250023/P1                                      
/// 6.SSR ADTK 1E BY KMG28AUG23/1742 OR CXL JD5324 Y06SEP                          
/// 7.SSR TKNE JD HK1 DXJPKX 5324 Y06SEP 8989198306578/1/P1                        
/// 8.SSR CHLD JD HK1 25MAY21/P1                                                   
/// 9.OSI JD CTCT13320512490                                                       
/// 10.OSI JD CTCM15718791505/P1                                                    
/// 11.OSI JD ADT/8989198306575    ";
/// if let Ok(info) = eterm_parser::parse_pat(text){
///     //assert_eq!(info.pnr_code, Some("KE9SWE"));
/// } else {
///     //assert_eq!(true, false);
/// }
/// ```
pub fn parse_pat(text: &str) -> anyhow::Result<pat::Pat> {
    pat::Pat::parse(text)
}