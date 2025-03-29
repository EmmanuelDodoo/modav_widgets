let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd ~/projects/modav/widgets
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +1 ~/projects/modav/widgets
badd +84 src/main.rs
badd +264 src/custom.rs
badd +6 .nvim/config.lua
badd +8 Cargo.toml
badd +8 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_highlighter-0.13.0/src/lib.rs
badd +10 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/syntect-5.2.0/src/highlighting/highlighter.rs
badd +61 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_core-0.13.2/src/text/highlighter.rs
badd +14 term://~/projects/modav/widgets//3552:/bin/bash
badd +1 term://~/projects/modav/widgets//3836:/bin/bash
badd +1 term://~/projects/modav/widgets//5582:/bin/bash
badd +3 term://~/projects/modav/widgets//6048:/bin/bash
badd +22 term://~/projects/modav/widgets//6531:/bin/bash
badd +44 term://~/projects/modav/widgets//6861:/bin/bash
badd +44 src/temp.txt
badd +1 term://~/projects/modav/widgets//7632:/bin/bash
badd +2 term://~/projects/modav/widgets//7853:/bin/bash
badd +1 term://~/projects/modav/widgets//8092:/bin/bash
badd +1 term://~/projects/modav/widgets//8330:/bin/bash
badd +15 term://~/projects/modav/widgets//8569:/bin/bash
badd +2 term://~/projects/modav/widgets//8879:/bin/bash
badd +1 term://~/projects/modav/widgets//9093:/bin/bash
badd +1 term://~/projects/modav/widgets//9590:/bin/bash
badd +1 term://~/projects/modav/widgets//9809:/bin/bash
badd +54 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/syntect-5.2.0/src/parsing/syntax_set.rs
badd +161 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/syntect-5.2.0/src/dumps.rs
badd +1 term://~/projects/modav/widgets//10920:/bin/bash
badd +52 term://~/projects/modav/widgets//11351:/bin/bash
badd +120 term://~/projects/modav/widgets//11581:/bin/bash
badd +44 term://~/projects/modav/widgets//11841:/bin/bash
badd +15 term://~/projects/modav/widgets//12093:/bin/bash
badd +1 term://~/projects/modav/widgets//12468:/bin/bash
badd +62 term://~/projects/modav/widgets//12680:/bin/bash
badd +44 term://~/projects/modav/widgets//14161:/bin/bash
badd +72 term://~/projects/modav/widgets//14402:/bin/bash
badd +44 term://~/projects/modav/widgets//14642:/bin/bash
badd +78 term://~/projects/modav/widgets//14880:/bin/bash
badd +97 term://~/projects/modav/widgets//15970:/bin/bash
badd +31 term://~/projects/modav/widgets//16213:/bin/bash
badd +63 term://~/projects/modav/widgets//16470:/bin/bash
badd +118 term://~/projects/modav/widgets//17988:/bin/bash
badd +15 term://~/projects/modav/widgets//18681:/bin/bash
badd +1 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_widget-0.13.4/src/text_editor.rs
badd +1250 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_widget-0.13.4/src/helpers.rs
badd +38 term://~/projects/modav/widgets//20217:/bin/bash
badd +42 term://~/projects/modav/widgets//22605:/bin/bash
badd +1 term://~/projects/modav/widgets//22833:/bin/bash
badd +1 term://~/projects/modav/widgets//23095:/bin/bash
badd +1 term://~/projects/modav/widgets//24273:/bin/bash
badd +1 term://~/projects/modav/widgets//24826:/bin/bash
badd +49 term://~/projects/modav/widgets//25226:/bin/bash
badd +1 src/lib.rs
badd +1 src/highlighter
badd +48 src/highlighter.rs
badd +55 term://~/projects/modav/widgets//11838:/bin/bash
badd +1 src/editor.rs
badd +186 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_core-0.13.2/src/text.rs
badd +9 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/iced_core-0.13.2/src/text/editor.rs
badd +24 term://~/projects/modav/widgets//29956:/bin/bash
badd +19 term://~/projects/modav/widgets//31690:/bin/bash
badd +221 src/table.rs
badd +12 term://~/projects/modav/widgets//2350:/bin/bash
badd +12 term://~/projects/modav/widgets//3223:/bin/bash
badd +12 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/size.rs
badd +1210 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/helpers.rs
badd +30 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text.rs
badd +60 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/widget/text.rs
badd +248 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/text.rs
badd +223 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/row.rs
badd +184 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/layout/flex.rs
badd +65 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/text/paragraph.rs
badd +14 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/layout.rs
badd +1463 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text_input.rs
badd +8 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text_input/editor.rs
badd +75 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text_input/cursor.rs
badd +14 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text_input/value.rs
badd +13 term://~/projects/modav/widgets//24676:/bin/bash
badd +95 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/text_editor.rs
badd +146 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/pick_list.rs
badd +1 term://~/projects/modav/widgets//1038:/bin/bash
badd +1 term://~/projects/modav/widgets//32594:/bin/bash
badd +1 term://~/projects/modav/widgets//858:/bin/bash
badd +17 term://~/projects/modav/widgets//1758:/bin/bash
badd +1 term://~/projects/modav/widgets//13266:/bin/bash
badd +17 term://~/projects/modav/widgets//13997:/bin/bash
badd +1 term://~/projects/modav/widgets//14595:/bin/bash
badd +1 term://~/projects/modav/widgets//27440:/bin/bash
badd +372 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/container.rs
badd +20 term://~/projects/modav/widgets//3094:/bin/bash
badd +90 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/font.rs
badd +36 term://~/projects/modav/widgets//4123:/bin/bash
badd +74 term://~/projects/modav/widgets//9040:/bin/bash
badd +1 term://~/projects/modav/widgets//20231:/bin/bash
badd +21 term://~/projects/modav/widgets//23453:/bin/bash
badd +1 term://~/projects/modav/widgets//1994:/bin/bash
badd +84 term://~/projects/modav/widgets//15615:/bin/bash
badd +10 term://~/projects/modav/widgets//27021:/bin/bash
badd +350 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_graphics-0.13.0/src/text/paragraph.rs
badd +204 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.12.1/src/buffer.rs
badd +119 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/layout/limits.rs
badd +1 term://~/projects/modav/widgets//7842:/bin/bash
badd +10 term://~/projects/modav/widgets//890:/bin/bash
badd +8 term://~/projects/modav/widgets//4131:/bin/bash
badd +1 term://~/projects/modav/widgets//11056:/bin/bash
badd +10 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/window/redraw_request.rs
badd +1 src/table/utils
badd +201 src/table/utils.rs
badd +9 term://~/projects/modav/widgets//6279:/bin/bash
badd +1 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/mouse.rs
badd +1 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/window/event.rs
badd +1 term://~/projects/modav/widgets//4787:/bin/bash
badd +30 term://~/projects/modav/widgets//17099:/bin/bash
badd +1 term://~/projects/modav/widgets//17819:/bin/bash
badd +420 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_widget-0.13.4/src/scrollable.rs
badd +1 term://~/projects/modav/widgets//7243:/bin/bash
badd +10 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/rectangle.rs
badd +2220 term://~/projects/modav/widgets//74067:/bin/bash
badd +3 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/vector.rs
badd +12 term://~/projects/modav/widgets//147160:/bin/bash
badd +1 term://~/projects/modav/widgets//155138:/bin/bash
badd +1 term://~/projects/modav/widgets//187273:/bin/bash
badd +1 term://~/projects/modav/widgets//201341:/bin/bash
badd +6 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/layout/node.rs
badd +1 term://~/projects/modav/widgets//215749:/bin/bash
badd +1 term://~/projects/modav/widgets//216774:/bin/bash
badd +3 term://~/projects/modav/widgets//253223:/bin/bash
badd +56 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/mouse/event.rs
badd +1 term://~/projects/modav/widgets//376950:/bin/bash
badd +1 term://~/projects/modav/widgets//389969:/bin/bash
badd +1 gitsigns:///home/custos/projects/modav/widgets/.git//:0:src/table.rs
badd +20 term://~/projects/modav/widgets//481809:/bin/bash
badd +32 term://~/projects/modav/widgets//507592:/bin/bash
badd +91 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_core-0.13.2/src/point.rs
badd +136 term://~/projects/modav/widgets//14017:/bin/bash
badd +404 term://~/projects/modav/widgets//25843:/bin/bash
badd +1 term://~/projects/modav/widgets//5670:/bin/bash
badd +2679 ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/string.rs
badd +1 .git
badd +52 ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iced_renderer-0.13.0/src/lib.rs
argglobal
%argdel
$argadd ~/projects/modav/widgets
edit src/table.rs
argglobal
balt src/main.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=99
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
21,24fold
38,47fold
50,61fold
63,66fold
68,71fold
73,76fold
78,81fold
83,86fold
88,91fold
49,92fold
95,97fold
99,101fold
103,105fold
110,119fold
107,120fold
136,138fold
140,147fold
151,158fold
150,159fold
160,168fold
149,169fold
131,170fold
122,170fold
180,182fold
186,189fold
179,190fold
172,190fold
207,217fold
205,218fold
202,219fold
192,219fold
94,220fold
226,228fold
222,229fold
231,262fold
291,293fold
291,295fold
291,295fold
296,297fold
299,303fold
307,310fold
312,315fold
317,320fold
322,325fold
327,331fold
278,365fold
369,371fold
368,373fold
375,377fold
380,383fold
385,392fold
394,396fold
398,407fold
409,411fold
421,427fold
421,428fold
421,429fold
448,456fold
459,462fold
465,468fold
469,475fold
446,483fold
496,497fold
496,498fold
496,499fold
489,500fold
446,506fold
446,506fold
440,507fold
510,513fold
515,521fold
440,524fold
440,526fold
440,526fold
440,526fold
440,527fold
532,534fold
532,536fold
532,536fold
436,540fold
562,565fold
558,578fold
583,584fold
586,588fold
586,591fold
586,591fold
558,594fold
558,594fold
557,595fold
557,603fold
557,603fold
553,606fold
609,610fold
609,611fold
609,612fold
608,618fold
622,623fold
622,624fold
622,625fold
628,629fold
620,630fold
632,634fold
632,635fold
632,636fold
632,637fold
639,641fold
639,642fold
639,643fold
639,644fold
649,652fold
654,659fold
413,661fold
664,666fold
673,674fold
673,675fold
672,676fold
672,678fold
672,678fold
684,690fold
683,697fold
706,711fold
715,717fold
726,727fold
737,739fold
735,740fold
663,743fold
746,748fold
749,755fold
762,768fold
771,772fold
745,791fold
794,796fold
801,807fold
793,812fold
829,831fold
829,833fold
829,833fold
836,838fold
836,840fold
836,840fold
843,846fold
852,855fold
857,858fold
859,862fold
857,862fold
865,868fold
865,869fold
824,886fold
814,886fold
897,899fold
897,901fold
897,901fold
903,910fold
912,919fold
902,920fold
895,921fold
894,922fold
888,922fold
935,942fold
943,950fold
934,951fold
932,952fold
961,968fold
969,976fold
960,977fold
958,978fold
930,979fold
924,979fold
993,1000fold
1001,1008fold
992,1009fold
989,1010fold
1016,1023fold
1024,1031fold
1015,1032fold
1012,1033fold
1039,1046fold
1047,1054fold
1038,1055fold
1035,1056fold
987,1057fold
981,1057fold
1067,1074fold
1076,1083fold
1066,1084fold
1065,1085fold
1059,1085fold
1096,1103fold
1095,1104fold
1109,1110fold
1109,1111fold
1115,1121fold
1127,1128fold
1127,1129fold
1127,1130fold
1133,1140fold
1142,1149fold
1132,1150fold
1126,1151fold
1123,1152fold
1154,1160fold
1167,1168fold
1167,1169fold
1167,1170fold
1173,1174fold
1173,1175fold
1176,1177fold
1176,1178fold
1181,1188fold
1180,1189fold
1192,1199fold
1191,1200fold
1202,1204fold
1212,1214fold
1211,1215fold
1206,1215fold
1166,1216fold
1164,1216fold
1218,1224fold
1229,1230fold
1229,1231fold
1229,1232fold
1235,1242fold
1245,1252fold
1254,1261fold
1244,1262fold
1234,1263fold
1271,1273fold
1270,1274fold
1265,1274fold
1227,1275fold
1288,1295fold
1287,1296fold
1285,1297fold
1308,1315fold
1307,1316fold
1305,1317fold
1277,1319fold
1094,1320fold
1087,1320fold
1331,1333fold
1331,1334fold
1344,1345fold
1347,1350fold
1353,1364fold
1352,1365fold
1352,1367fold
1352,1367fold
1343,1370fold
1375,1376fold
1378,1379fold
1384,1395fold
1396,1398fold
1396,1400fold
1396,1400fold
1371,1403fold
1342,1404fold
1335,1405fold
1335,1407fold
1331,1407fold
1412,1413fold
1419,1421fold
1416,1422fold
1415,1423fold
1415,1425fold
1415,1425fold
1409,1426fold
1429,1431fold
1428,1432fold
1428,1434fold
1428,1434fold
1330,1435fold
1322,1435fold
1451,1452fold
1451,1453fold
1458,1462fold
1456,1472fold
1474,1476fold
1480,1484fold
1487,1494fold
1486,1495fold
1446,1496fold
1437,1496fold
1504,1505fold
1504,1506fold
1507,1508fold
1507,1509fold
1512,1513fold
1512,1514fold
1512,1515fold
1519,1520fold
1519,1521fold
1519,1522fold
1519,1523fold
1527,1529fold
1532,1540fold
1541,1548fold
1550,1552fold
1554,1556fold
1558,1560fold
1531,1561fold
1511,1562fold
1568,1569fold
1568,1570fold
1568,1571fold
1568,1572fold
1574,1576fold
1579,1587fold
1588,1595fold
1597,1599fold
1601,1603fold
1605,1607fold
1578,1608fold
1566,1609fold
1502,1612fold
1498,1612fold
1621,1622fold
1621,1623fold
1625,1627fold
1629,1630fold
1629,1631fold
1633,1635fold
1637,1638fold
1637,1639fold
1641,1643fold
1618,1646fold
1614,1646fold
1656,1657fold
1656,1658fold
1660,1662fold
1664,1665fold
1664,1666fold
1667,1669fold
1652,1672fold
1648,1672fold
1679,1681fold
1685,1686fold
1685,1687fold
1688,1690fold
1695,1696fold
1695,1697fold
1698,1700fold
1702,1703fold
1702,1704fold
1705,1707fold
1694,1708fold
1678,1711fold
1674,1711fold
1724,1726fold
1732,1733fold
1732,1734fold
1735,1736fold
1735,1737fold
1735,1738fold
1735,1739fold
1740,1741fold
1740,1742fold
1740,1743fold
1740,1744fold
1747,1748fold
1747,1749fold
1754,1755fold
1754,1756fold
1754,1757fold
1754,1758fold
1754,1759fold
1753,1760fold
1761,1762fold
1761,1763fold
1753,1764fold
1753,1764fold
1766,1768fold
1766,1773fold
1766,1773fold
1778,1781fold
1785,1787fold
1791,1793fold
1791,1794fold
1789,1799fold
1805,1807fold
1805,1808fold
1789,1813fold
1789,1813fold
1816,1820fold
1815,1823fold
1825,1829fold
1833,1835fold
1833,1837fold
1833,1837fold
1833,1838fold
1841,1842fold
1840,1843fold
1840,1845fold
1840,1845fold
1832,1848fold
1851,1852fold
1850,1852fold
1849,1856fold
1857,1860fold
1831,1861fold
1864,1868fold
1751,1871fold
1872,1876fold
1747,1877fold
1730,1878fold
1889,1891fold
1884,1891fold
1895,1896fold
1895,1897fold
1895,1898fold
1899,1900fold
1899,1901fold
1899,1902fold
1905,1906fold
1905,1907fold
1905,1909fold
1905,1911fold
1905,1912fold
1905,1913fold
1904,1916fold
1917,1918fold
1917,1919fold
1921,1922fold
1921,1923fold
1921,1924fold
1921,1925fold
1920,1926fold
1917,1926fold
1917,1927fold
1904,1930fold
1904,1930fold
1933,1937fold
1932,1940fold
1942,1943fold
1945,1946fold
1882,1949fold
1954,1956fold
1953,1967fold
1969,1971fold
1979,1981fold
1973,1981fold
1989,1991fold
1989,1992fold
1999,2001fold
1999,2002fold
1994,2005fold
1987,2005fold
2009,2012fold
2020,2026fold
2033,2037fold
2016,2040fold
2015,2041fold
2043,2044fold
2043,2045fold
2043,2046fold
2051,2057fold
2067,2071fold
2066,2071fold
2047,2073fold
2043,2073fold
2042,2074fold
2014,2076fold
2080,2082fold
2087,2093fold
2083,2096fold
2080,2097fold
2080,2097fold
2079,2102fold
2107,2113fold
2103,2117fold
2122,2128fold
2118,2132fold
2134,2136fold
2134,2138fold
2134,2138fold
2133,2141fold
2143,2145fold
2143,2147fold
2143,2147fold
2142,2150fold
2151,2154fold
2159,2161fold
2078,2164fold
1968,2165fold
1728,2167fold
1723,2168fold
1713,2168fold
2181,2182fold
2181,2183fold
2185,2190fold
2192,2193fold
2192,2194fold
2197,2198fold
2197,2199fold
2197,2200fold
2197,2201fold
2202,2204fold
2197,2204fold
2206,2208fold
2206,2209fold
2213,2223fold
2211,2226fold
2196,2231fold
2233,2234fold
2233,2235fold
2237,2242fold
2178,2245fold
2177,2247fold
2176,2248fold
2170,2248fold
2273,2277fold
2270,2280fold
2282,2286fold
2293,2299fold
2290,2300fold
2290,2302fold
2290,2302fold
2290,2303fold
2306,2309fold
2305,2310fold
2305,2312fold
2305,2312fold
2289,2314fold
2315,2319fold
2320,2323fold
2288,2324fold
2269,2331fold
2338,2342fold
2343,2347fold
2337,2349fold
2335,2350fold
2332,2353fold
2268,2354fold
2267,2355fold
2363,2367fold
2362,2370fold
2374,2375fold
2377,2378fold
2359,2381fold
2383,2385fold
2392,2395fold
2398,2399fold
2398,2400fold
2398,2401fold
2410,2412fold
2408,2414fold
2416,2422fold
2402,2427fold
2398,2427fold
2397,2428fold
2432,2438fold
2432,2444fold
2432,2444fold
2431,2445fold
2449,2455fold
2446,2457fold
2461,2467fold
2458,2469fold
2471,2473fold
2471,2475fold
2471,2475fold
2470,2477fold
2479,2481fold
2479,2483fold
2479,2483fold
2478,2485fold
2486,2489fold
2494,2496fold
2430,2499fold
2382,2502fold
2265,2504fold
2257,2505fold
2250,2505fold
2522,2523fold
2522,2524fold
2526,2527fold
2526,2528fold
2530,2531fold
2530,2532fold
2543,2547fold
2540,2548fold
2539,2549fold
2539,2551fold
2539,2551fold
2555,2556fold
2555,2557fold
2558,2559fold
2558,2560fold
2563,2568fold
2562,2574fold
2575,2584fold
2553,2585fold
2587,2590fold
2592,2594fold
2605,2606fold
2605,2607fold
2605,2608fold
2604,2609fold
2616,2622fold
2612,2625fold
2604,2626fold
2604,2626fold
2602,2631fold
2632,2635fold
2596,2636fold
2538,2637fold
2640,2643fold
2649,2651fold
2654,2655fold
2654,2656fold
2657,2658fold
2657,2659fold
2662,2667fold
2670,2671fold
2661,2674fold
2675,2684fold
2652,2685fold
2648,2687fold
2647,2688fold
2694,2695fold
2694,2696fold
2697,2698fold
2697,2699fold
2702,2707fold
2701,2713fold
2714,2723fold
2692,2724fold
2726,2731fold
2734,2735fold
2734,2736fold
2737,2738fold
2737,2739fold
2742,2747fold
2741,2753fold
2725,2758fold
2760,2762fold
2765,2766fold
2765,2767fold
2768,2769fold
2768,2770fold
2773,2778fold
2772,2784fold
2785,2794fold
2763,2795fold
2759,2797fold
2798,2800fold
2802,2804fold
2801,2805fold
2807,2812fold
2806,2813fold
2819,2820fold
2822,2824fold
2816,2825fold
2815,2826fold
2814,2827fold
2536,2829fold
2518,2832fold
2507,2832fold
264,2833fold
2835,2838fold
2844,2850fold
2841,2851fold
2868,2874fold
2875,2884fold
2864,2885fold
2863,2886fold
2853,2886fold
2840,2887fold
2890,2894fold
2897,2904fold
2912,2924fold
2906,2924fold
2938,2942fold
2944,2948fold
2935,2951fold
2926,2951fold
2954,2964fold
2953,2965fold
2968,2971fold
2967,2972fold
2984,2986fold
2988,2990fold
2992,2994fold
2974,2999fold
3006,3008fold
3009,3013fold
3006,3014fold
3006,3014fold
3005,3015fold
3001,3015fold
3022,3023fold
3022,3024fold
3021,3029fold
3017,3029fold
3035,3038fold
3032,3043fold
3032,3045fold
3032,3045fold
3031,3046fold
3058,3059fold
3058,3060fold
3058,3061fold
3054,3066fold
3048,3066fold
3069,3071fold
3076,3078fold
3083,3085fold
3087,3089fold
3091,3093fold
3068,3096fold
3102,3104fold
3106,3108fold
3110,3112fold
3109,3115fold
3100,3116fold
3099,3117fold
let &fdl = &fdl
21
normal! zc
38
normal! zc
49
normal! zo
50
normal! zc
63
normal! zc
68
normal! zc
73
normal! zc
78
normal! zc
83
normal! zc
88
normal! zc
49
normal! zc
94
normal! zo
95
normal! zc
99
normal! zc
103
normal! zc
107
normal! zo
110
normal! zc
107
normal! zc
122
normal! zo
131
normal! zo
136
normal! zc
140
normal! zc
149
normal! zo
150
normal! zo
151
normal! zc
150
normal! zc
160
normal! zc
149
normal! zc
131
normal! zc
122
normal! zc
172
normal! zo
179
normal! zo
180
normal! zc
186
normal! zc
179
normal! zc
172
normal! zc
192
normal! zo
202
normal! zo
205
normal! zo
207
normal! zc
205
normal! zc
202
normal! zc
192
normal! zc
94
normal! zc
222
normal! zo
226
normal! zc
222
normal! zc
231
normal! zc
264
normal! zo
278
normal! zo
291
normal! zo
291
normal! zo
291
normal! zc
291
normal! zc
291
normal! zc
296
normal! zc
299
normal! zc
307
normal! zc
312
normal! zc
317
normal! zc
322
normal! zc
327
normal! zc
278
normal! zc
368
normal! zo
369
normal! zc
368
normal! zc
375
normal! zc
380
normal! zc
385
normal! zc
394
normal! zc
398
normal! zc
409
normal! zc
413
normal! zo
421
normal! zo
421
normal! zo
421
normal! zc
421
normal! zc
421
normal! zc
436
normal! zo
440
normal! zo
440
normal! zo
440
normal! zo
440
normal! zo
440
normal! zo
440
normal! zo
446
normal! zo
446
normal! zo
446
normal! zo
448
normal! zc
459
normal! zc
465
normal! zc
469
normal! zc
446
normal! zc
489
normal! zo
496
normal! zo
496
normal! zo
496
normal! zc
496
normal! zc
496
normal! zc
489
normal! zc
446
normal! zc
446
normal! zc
440
normal! zc
510
normal! zc
515
normal! zc
440
normal! zc
440
normal! zc
440
normal! zc
440
normal! zc
440
normal! zc
532
normal! zo
532
normal! zo
532
normal! zc
532
normal! zc
532
normal! zc
436
normal! zc
553
normal! zo
557
normal! zo
557
normal! zo
557
normal! zo
558
normal! zo
558
normal! zo
558
normal! zo
562
normal! zc
558
normal! zc
583
normal! zc
586
normal! zo
586
normal! zo
586
normal! zc
586
normal! zc
586
normal! zc
558
normal! zc
558
normal! zc
557
normal! zc
557
normal! zc
557
normal! zc
553
normal! zc
608
normal! zo
609
normal! zo
609
normal! zo
609
normal! zc
609
normal! zc
609
normal! zc
608
normal! zc
620
normal! zo
622
normal! zo
622
normal! zo
622
normal! zc
622
normal! zc
622
normal! zc
628
normal! zc
620
normal! zc
632
normal! zo
632
normal! zo
632
normal! zo
632
normal! zc
632
normal! zc
632
normal! zc
632
normal! zc
639
normal! zo
639
normal! zo
639
normal! zo
639
normal! zc
639
normal! zc
639
normal! zc
639
normal! zc
649
normal! zc
654
normal! zc
413
normal! zc
663
normal! zo
664
normal! zc
672
normal! zo
672
normal! zo
672
normal! zo
673
normal! zo
673
normal! zc
673
normal! zc
672
normal! zc
672
normal! zc
672
normal! zc
683
normal! zo
684
normal! zc
683
normal! zc
706
normal! zc
715
normal! zc
726
normal! zc
735
normal! zo
737
normal! zc
735
normal! zc
663
normal! zc
745
normal! zo
746
normal! zc
749
normal! zc
762
normal! zc
771
normal! zc
745
normal! zc
793
normal! zo
794
normal! zc
801
normal! zc
793
normal! zc
814
normal! zo
824
normal! zo
829
normal! zo
829
normal! zo
829
normal! zc
829
normal! zc
829
normal! zc
836
normal! zo
836
normal! zo
836
normal! zc
836
normal! zc
836
normal! zc
843
normal! zc
852
normal! zc
857
normal! zo
857
normal! zc
859
normal! zc
857
normal! zc
865
normal! zo
865
normal! zc
865
normal! zc
824
normal! zc
814
normal! zc
888
normal! zo
894
normal! zo
895
normal! zo
897
normal! zo
897
normal! zo
897
normal! zc
897
normal! zc
897
normal! zc
902
normal! zo
903
normal! zc
912
normal! zc
902
normal! zc
895
normal! zc
894
normal! zc
888
normal! zc
924
normal! zo
930
normal! zo
932
normal! zo
934
normal! zo
935
normal! zc
943
normal! zc
934
normal! zc
932
normal! zc
958
normal! zo
960
normal! zo
961
normal! zc
969
normal! zc
960
normal! zc
958
normal! zc
930
normal! zc
924
normal! zc
981
normal! zo
987
normal! zo
989
normal! zo
992
normal! zo
993
normal! zc
1001
normal! zc
992
normal! zc
989
normal! zc
1012
normal! zo
1015
normal! zo
1016
normal! zc
1024
normal! zc
1015
normal! zc
1012
normal! zc
1035
normal! zo
1038
normal! zo
1039
normal! zc
1047
normal! zc
1038
normal! zc
1035
normal! zc
987
normal! zc
981
normal! zc
1059
normal! zo
1065
normal! zo
1066
normal! zo
1067
normal! zc
1076
normal! zc
1066
normal! zc
1065
normal! zc
1059
normal! zc
1087
normal! zo
1094
normal! zo
1095
normal! zo
1096
normal! zc
1095
normal! zc
1109
normal! zo
1109
normal! zc
1109
normal! zc
1115
normal! zc
1123
normal! zo
1126
normal! zo
1127
normal! zo
1127
normal! zo
1127
normal! zc
1127
normal! zc
1127
normal! zc
1132
normal! zo
1133
normal! zc
1142
normal! zc
1132
normal! zc
1126
normal! zc
1123
normal! zc
1154
normal! zc
1164
normal! zo
1166
normal! zo
1167
normal! zo
1167
normal! zo
1167
normal! zc
1167
normal! zc
1167
normal! zc
1173
normal! zo
1173
normal! zc
1173
normal! zc
1176
normal! zo
1176
normal! zc
1176
normal! zc
1180
normal! zo
1181
normal! zc
1180
normal! zc
1191
normal! zo
1192
normal! zc
1191
normal! zc
1202
normal! zc
1206
normal! zo
1211
normal! zo
1212
normal! zc
1211
normal! zc
1206
normal! zc
1166
normal! zc
1164
normal! zc
1218
normal! zc
1227
normal! zo
1229
normal! zo
1229
normal! zo
1229
normal! zc
1229
normal! zc
1229
normal! zc
1234
normal! zo
1235
normal! zc
1244
normal! zo
1245
normal! zc
1254
normal! zc
1244
normal! zc
1234
normal! zc
1265
normal! zo
1270
normal! zo
1271
normal! zc
1270
normal! zc
1265
normal! zc
1227
normal! zc
1277
normal! zo
1285
normal! zo
1287
normal! zo
1288
normal! zc
1287
normal! zc
1285
normal! zc
1305
normal! zo
1307
normal! zo
1308
normal! zc
1307
normal! zc
1305
normal! zc
1277
normal! zc
1094
normal! zc
1087
normal! zc
1322
normal! zo
1330
normal! zo
1331
normal! zo
1331
normal! zo
1331
normal! zc
1331
normal! zc
1335
normal! zo
1335
normal! zo
1342
normal! zo
1343
normal! zo
1344
normal! zc
1347
normal! zc
1352
normal! zo
1352
normal! zo
1352
normal! zo
1353
normal! zc
1352
normal! zc
1352
normal! zc
1352
normal! zc
1343
normal! zc
1371
normal! zo
1375
normal! zc
1378
normal! zc
1384
normal! zc
1396
normal! zo
1396
normal! zo
1396
normal! zc
1396
normal! zc
1396
normal! zc
1371
normal! zc
1342
normal! zc
1335
normal! zc
1335
normal! zc
1331
normal! zc
1409
normal! zo
1412
normal! zc
1415
normal! zo
1415
normal! zo
1415
normal! zo
1416
normal! zo
1419
normal! zc
1416
normal! zc
1415
normal! zc
1415
normal! zc
1415
normal! zc
1409
normal! zc
1428
normal! zo
1428
normal! zo
1428
normal! zo
1429
normal! zc
1428
normal! zc
1428
normal! zc
1428
normal! zc
1330
normal! zc
1322
normal! zc
1437
normal! zo
1446
normal! zo
1451
normal! zo
1451
normal! zc
1451
normal! zc
1456
normal! zo
1458
normal! zc
1456
normal! zc
1474
normal! zc
1480
normal! zc
1486
normal! zo
1487
normal! zc
1486
normal! zc
1446
normal! zc
1437
normal! zc
1498
normal! zo
1502
normal! zo
1504
normal! zo
1504
normal! zc
1504
normal! zc
1507
normal! zo
1507
normal! zc
1507
normal! zc
1511
normal! zo
1512
normal! zo
1512
normal! zo
1512
normal! zc
1512
normal! zc
1512
normal! zc
1519
normal! zo
1519
normal! zo
1519
normal! zo
1519
normal! zc
1519
normal! zc
1519
normal! zc
1519
normal! zc
1527
normal! zc
1531
normal! zo
1532
normal! zc
1541
normal! zc
1550
normal! zc
1554
normal! zc
1558
normal! zc
1531
normal! zc
1511
normal! zc
1566
normal! zo
1568
normal! zo
1568
normal! zo
1568
normal! zo
1568
normal! zc
1568
normal! zc
1568
normal! zc
1568
normal! zc
1574
normal! zc
1578
normal! zo
1579
normal! zc
1588
normal! zc
1597
normal! zc
1601
normal! zc
1605
normal! zc
1578
normal! zc
1566
normal! zc
1502
normal! zc
1498
normal! zc
1614
normal! zo
1618
normal! zo
1621
normal! zo
1621
normal! zc
1621
normal! zc
1625
normal! zc
1629
normal! zo
1629
normal! zc
1629
normal! zc
1633
normal! zc
1637
normal! zo
1637
normal! zc
1637
normal! zc
1641
normal! zc
1618
normal! zc
1614
normal! zc
1648
normal! zo
1652
normal! zo
1656
normal! zo
1656
normal! zc
1656
normal! zc
1660
normal! zc
1664
normal! zo
1664
normal! zc
1664
normal! zc
1667
normal! zc
1652
normal! zc
1648
normal! zc
1674
normal! zo
1678
normal! zo
1679
normal! zc
1685
normal! zo
1685
normal! zc
1685
normal! zc
1688
normal! zc
1694
normal! zo
1695
normal! zo
1695
normal! zc
1695
normal! zc
1698
normal! zc
1702
normal! zo
1702
normal! zc
1702
normal! zc
1705
normal! zc
1694
normal! zc
1678
normal! zc
1674
normal! zc
1713
normal! zo
1723
normal! zo
1724
normal! zc
1728
normal! zo
1730
normal! zo
1732
normal! zo
1732
normal! zc
1732
normal! zc
1735
normal! zo
1735
normal! zo
1735
normal! zo
1735
normal! zc
1735
normal! zc
1735
normal! zc
1735
normal! zc
1740
normal! zo
1740
normal! zo
1740
normal! zo
1740
normal! zc
1740
normal! zc
1740
normal! zc
1740
normal! zc
1747
normal! zo
1747
normal! zo
1747
normal! zc
1747
normal! zc
1751
normal! zo
1753
normal! zo
1753
normal! zo
1753
normal! zo
1754
normal! zo
1754
normal! zo
1754
normal! zo
1754
normal! zo
1754
normal! zc
1754
normal! zc
1754
normal! zc
1754
normal! zc
1754
normal! zc
1753
normal! zc
1761
normal! zo
1761
normal! zc
1761
normal! zc
1753
normal! zc
1753
normal! zc
1766
normal! zo
1766
normal! zo
1766
normal! zc
1766
normal! zc
1766
normal! zc
1778
normal! zc
1785
normal! zc
1789
normal! zo
1789
normal! zo
1789
normal! zo
1791
normal! zo
1791
normal! zc
1791
normal! zc
1789
normal! zc
1805
normal! zo
1805
normal! zc
1805
normal! zc
1789
normal! zc
1789
normal! zc
1815
normal! zo
1816
normal! zc
1815
normal! zc
1825
normal! zc
1831
normal! zo
1832
normal! zo
1833
normal! zo
1833
normal! zo
1833
normal! zo
1833
normal! zc
1833
normal! zc
1833
normal! zc
1833
normal! zc
1840
normal! zo
1840
normal! zo
1840
normal! zo
1841
normal! zc
1840
normal! zc
1840
normal! zc
1840
normal! zc
1832
normal! zc
1849
normal! zo
1850
normal! zo
1851
normal! zc
1850
normal! zc
1849
normal! zc
1857
normal! zc
1831
normal! zc
1864
normal! zc
1751
normal! zc
1872
normal! zc
1747
normal! zc
1730
normal! zc
1882
normal! zo
1884
normal! zo
1889
normal! zc
1884
normal! zc
1895
normal! zo
1895
normal! zo
1895
normal! zc
1895
normal! zc
1895
normal! zc
1899
normal! zo
1899
normal! zo
1899
normal! zc
1899
normal! zc
1899
normal! zc
1904
normal! zo
1904
normal! zo
1904
normal! zo
1905
normal! zo
1905
normal! zo
1905
normal! zo
1905
normal! zo
1905
normal! zo
1905
normal! zc
1905
normal! zc
1905
normal! zc
1905
normal! zc
1905
normal! zc
1905
normal! zc
1904
normal! zc
1917
normal! zo
1917
normal! zo
1917
normal! zo
1917
normal! zc
1917
normal! zc
1920
normal! zo
1921
normal! zo
1921
normal! zo
1921
normal! zo
1921
normal! zc
1921
normal! zc
1921
normal! zc
1921
normal! zc
1920
normal! zc
1917
normal! zc
1917
normal! zc
1904
normal! zc
1904
normal! zc
1932
normal! zo
1933
normal! zc
1932
normal! zc
1942
normal! zc
1945
normal! zc
1882
normal! zc
1953
normal! zo
1954
normal! zc
1953
normal! zc
1968
normal! zo
1969
normal! zc
1973
normal! zo
1979
normal! zc
1973
normal! zc
1987
normal! zo
1989
normal! zo
1989
normal! zc
1989
normal! zc
1994
normal! zo
1999
normal! zo
1999
normal! zc
1999
normal! zc
1994
normal! zc
1987
normal! zc
2009
normal! zc
2014
normal! zo
2015
normal! zo
2016
normal! zo
2020
normal! zc
2033
normal! zc
2016
normal! zc
2015
normal! zc
2042
normal! zo
2043
normal! zo
2043
normal! zo
2043
normal! zo
2043
normal! zc
2043
normal! zc
2043
normal! zc
2047
normal! zo
2051
normal! zc
2066
normal! zo
2067
normal! zc
2066
normal! zc
2047
normal! zc
2043
normal! zc
2042
normal! zc
2014
normal! zc
2078
normal! zo
2079
normal! zo
2080
normal! zo
2080
normal! zo
2080
normal! zc
2083
normal! zo
2087
normal! zc
2083
normal! zc
2080
normal! zc
2080
normal! zc
2079
normal! zc
2103
normal! zo
2107
normal! zc
2103
normal! zc
2118
normal! zo
2122
normal! zc
2118
normal! zc
2133
normal! zo
2134
normal! zo
2134
normal! zo
2134
normal! zc
2134
normal! zc
2134
normal! zc
2133
normal! zc
2142
normal! zo
2143
normal! zo
2143
normal! zo
2143
normal! zc
2143
normal! zc
2143
normal! zc
2142
normal! zc
2151
normal! zc
2159
normal! zc
2078
normal! zc
1968
normal! zc
1728
normal! zc
1723
normal! zc
1713
normal! zc
2170
normal! zo
2176
normal! zo
2177
normal! zo
2178
normal! zo
2181
normal! zo
2181
normal! zc
2181
normal! zc
2185
normal! zc
2192
normal! zo
2192
normal! zc
2192
normal! zc
2196
normal! zo
2197
normal! zo
2197
normal! zo
2197
normal! zo
2197
normal! zo
2197
normal! zc
2197
normal! zc
2197
normal! zc
2197
normal! zc
2202
normal! zc
2197
normal! zc
2206
normal! zo
2206
normal! zc
2206
normal! zc
2211
normal! zo
2213
normal! zc
2211
normal! zc
2196
normal! zc
2233
normal! zo
2233
normal! zc
2233
normal! zc
2237
normal! zc
2178
normal! zc
2177
normal! zc
2176
normal! zc
2170
normal! zc
2250
normal! zo
2257
normal! zo
2265
normal! zo
2267
normal! zo
2268
normal! zo
2269
normal! zo
2270
normal! zo
2273
normal! zc
2270
normal! zc
2282
normal! zc
2288
normal! zo
2289
normal! zo
2290
normal! zo
2290
normal! zo
2290
normal! zo
2290
normal! zo
2293
normal! zc
2290
normal! zc
2290
normal! zc
2290
normal! zc
2290
normal! zc
2305
normal! zo
2305
normal! zo
2305
normal! zo
2306
normal! zc
2305
normal! zc
2305
normal! zc
2305
normal! zc
2289
normal! zc
2315
normal! zc
2320
normal! zc
2288
normal! zc
2269
normal! zc
2332
normal! zo
2335
normal! zo
2337
normal! zo
2338
normal! zc
2343
normal! zc
2337
normal! zc
2335
normal! zc
2332
normal! zc
2268
normal! zc
2267
normal! zc
2359
normal! zo
2362
normal! zo
2363
normal! zc
2362
normal! zc
2374
normal! zc
2377
normal! zc
2359
normal! zc
2382
normal! zo
2383
normal! zc
2392
normal! zc
2397
normal! zo
2398
normal! zo
2398
normal! zo
2398
normal! zo
2398
normal! zc
2398
normal! zc
2398
normal! zc
2402
normal! zo
2408
normal! zo
2410
normal! zc
2408
normal! zc
2416
normal! zc
2402
normal! zc
2398
normal! zc
2397
normal! zc
2430
normal! zo
2431
normal! zo
2432
normal! zo
2432
normal! zo
2432
normal! zc
2432
normal! zc
2432
normal! zc
2431
normal! zc
2446
normal! zo
2449
normal! zc
2446
normal! zc
2458
normal! zo
2461
normal! zc
2458
normal! zc
2470
normal! zo
2471
normal! zo
2471
normal! zo
2471
normal! zc
2471
normal! zc
2471
normal! zc
2470
normal! zc
2478
normal! zo
2479
normal! zo
2479
normal! zo
2479
normal! zc
2479
normal! zc
2479
normal! zc
2478
normal! zc
2486
normal! zc
2494
normal! zc
2430
normal! zc
2382
normal! zc
2265
normal! zc
2257
normal! zc
2250
normal! zc
2507
normal! zo
2518
normal! zo
2522
normal! zo
2522
normal! zc
2522
normal! zc
2526
normal! zo
2526
normal! zc
2526
normal! zc
2530
normal! zo
2530
normal! zc
2530
normal! zc
2536
normal! zo
2538
normal! zo
2539
normal! zo
2539
normal! zo
2539
normal! zo
2540
normal! zo
2543
normal! zc
2540
normal! zc
2539
normal! zc
2539
normal! zc
2539
normal! zc
2553
normal! zo
2555
normal! zo
2555
normal! zc
2555
normal! zc
2558
normal! zo
2558
normal! zc
2558
normal! zc
2562
normal! zo
2563
normal! zc
2562
normal! zc
2575
normal! zc
2553
normal! zc
2587
normal! zc
2592
normal! zc
2596
normal! zo
2602
normal! zo
2604
normal! zo
2604
normal! zo
2604
normal! zo
2605
normal! zo
2605
normal! zo
2605
normal! zc
2605
normal! zc
2605
normal! zc
2604
normal! zc
2612
normal! zo
2616
normal! zc
2612
normal! zc
2604
normal! zc
2604
normal! zc
2602
normal! zc
2632
normal! zc
2596
normal! zc
2538
normal! zc
2640
normal! zc
2647
normal! zo
2648
normal! zo
2649
normal! zc
2652
normal! zo
2654
normal! zo
2654
normal! zc
2654
normal! zc
2657
normal! zo
2657
normal! zc
2657
normal! zc
2661
normal! zo
2662
normal! zc
2670
normal! zc
2661
normal! zc
2675
normal! zc
2652
normal! zc
2648
normal! zc
2647
normal! zc
2692
normal! zo
2694
normal! zo
2694
normal! zc
2694
normal! zc
2697
normal! zo
2697
normal! zc
2697
normal! zc
2701
normal! zo
2702
normal! zc
2701
normal! zc
2714
normal! zc
2692
normal! zc
2725
normal! zo
2726
normal! zc
2734
normal! zo
2734
normal! zc
2734
normal! zc
2737
normal! zo
2737
normal! zc
2737
normal! zc
2741
normal! zo
2742
normal! zc
2741
normal! zc
2725
normal! zc
2759
normal! zo
2760
normal! zc
2763
normal! zo
2765
normal! zo
2765
normal! zc
2765
normal! zc
2768
normal! zo
2768
normal! zc
2768
normal! zc
2772
normal! zo
2773
normal! zc
2772
normal! zc
2785
normal! zc
2763
normal! zc
2759
normal! zc
2798
normal! zc
2801
normal! zo
2802
normal! zc
2801
normal! zc
2806
normal! zo
2807
normal! zc
2806
normal! zc
2814
normal! zo
2815
normal! zo
2816
normal! zo
2819
normal! zc
2822
normal! zc
2816
normal! zc
2815
normal! zc
2814
normal! zc
2536
normal! zc
2518
normal! zc
2507
normal! zc
264
normal! zc
2835
normal! zc
2840
normal! zo
2841
normal! zo
2844
normal! zc
2841
normal! zc
2853
normal! zo
2863
normal! zo
2864
normal! zo
2868
normal! zc
2875
normal! zc
2864
normal! zc
2863
normal! zc
2853
normal! zc
2840
normal! zc
2890
normal! zc
2897
normal! zc
2906
normal! zo
2912
normal! zc
2906
normal! zc
2926
normal! zo
2935
normal! zo
2938
normal! zc
2944
normal! zc
2935
normal! zc
2926
normal! zc
2953
normal! zo
2954
normal! zc
2953
normal! zc
2967
normal! zo
2968
normal! zc
2967
normal! zc
2974
normal! zo
2984
normal! zc
2988
normal! zc
2992
normal! zc
2974
normal! zc
3001
normal! zo
3005
normal! zo
3006
normal! zo
3006
normal! zo
3006
normal! zc
3009
normal! zc
3006
normal! zc
3006
normal! zc
3005
normal! zc
3001
normal! zc
3017
normal! zo
3021
normal! zo
3022
normal! zo
3022
normal! zc
3022
normal! zc
3021
normal! zc
3017
normal! zc
3031
normal! zo
3032
normal! zo
3032
normal! zo
3032
normal! zo
3035
normal! zc
3032
normal! zc
3032
normal! zc
3032
normal! zc
3031
normal! zc
3048
normal! zo
3054
normal! zo
3058
normal! zo
3058
normal! zo
3058
normal! zc
3058
normal! zc
3058
normal! zc
3054
normal! zc
3048
normal! zc
3068
normal! zo
3069
normal! zc
3076
normal! zc
3083
normal! zc
3087
normal! zc
3091
normal! zc
3068
normal! zc
3099
normal! zo
3100
normal! zo
3102
normal! zc
3106
normal! zc
3109
normal! zo
3110
normal! zc
3109
normal! zc
3100
normal! zc
3099
normal! zc
let s:l = 32 - ((17 * winheight(0) + 26) / 52)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 32
normal! 0
lcd ~/projects/modav/widgets
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
