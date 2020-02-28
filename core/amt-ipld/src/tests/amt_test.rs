use matches::matches;

use super::*;

#[test]
fn one_root_node() {
    // just one root node
    let bs = db();
    let mut root = Amt::new(bs);
    root.set(0, "0").unwrap();
    root.set(1, "1").unwrap();
    root.set(7, "7").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacedj2lbq4w2xr74jm4ndynfn66z2v2ybcl5lfmoeilezaqcq2pplui"
    );
}

#[test]
fn one_root_node_reorder_insert() {
    // reorder insert for one root node
    let bs = db();
    let mut root = Amt::new(bs);
    root.set(7, "7").unwrap();
    root.set(5, "5").unwrap();
    root.set(0, "0").unwrap();
    root.set(2, "2").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacecfpqjvhbe4sbanu4bjy6aws3qupk2y2h5hsr7fbxky7wbu6rtedi"
    );

    let mut collection = vec![];
    let _ = root.for_each(&mut |key, value| {
        collection.push((key, value.clone()));
        Ok(())
    });
    let m = vec![0, 2, 5, 7];
    for (src, (key, value)) in m.iter().zip(collection.into_iter()) {
        assert_eq!(*src, key);
        assert_eq!(Value::Text(src.to_string()), value);
    }

    for (src, (key, _)) in m.iter().zip(root.iter()) {
        assert_eq!(*src, key)
    }
}

#[test]
fn tow_level_node() {
    let bs = db();
    let mut root = Amt::new(bs);
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    root.set(8, "8").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );

    let bs = db();
    let mut root = Amt::new(bs);
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let _ = root.flush().unwrap();
    root.set(8, "8").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );
}

#[test]
fn tow_level_node_reorder_insert() {
    let bs = db();
    let mut root = Amt::new(bs);
    root.set(8, "8").unwrap();
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );

    let mut collection = vec![];
    let _ = root.for_each(&mut |key, value| {
        collection.push((key, value.clone()));
        Ok(())
    });
    let m = vec![1, 7, 8];
    for (src, (key, value)) in m.iter().zip(collection.into_iter()) {
        assert_eq!(*src, key);
        assert_eq!(Value::Text(src.to_string()), value);
    }

    let bs = db();
    let mut root = Amt::new(bs);
    root.set(8, "8").unwrap();
    let _ = root.flush().unwrap();
    root.set(7, "7").unwrap();
    root.set(1, "1").unwrap();
    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzaceazvpi5k466hzkiuypsbzrr65smq72fhwumnehb2mg6ixanbbttag"
    );

    let mut collection = vec![];
    let _ = root.for_each(&mut |key, value| {
        collection.push((key, value.clone()));
        Ok(())
    });
    let m = vec![1, 7, 8];
    for (src, (key, value)) in m.iter().zip(collection.into_iter()) {
        assert_eq!(*src, key);
        assert_eq!(Value::Text(src.to_string()), value);
    }

    for (src, (key, _)) in m.iter().zip(root.iter()) {
        assert_eq!(*src, key)
    }
}

#[test]
fn there_level() {
    use rand::seq::SliceRandom;
    let mut m = (0..65).collect::<Vec<_>>();
    let mut rng = rand::thread_rng();
    m.shuffle(&mut rng);

    let bs = db();
    let mut root = Amt::new(bs);

    for i in m.iter() {
        root.set(*i, *i).unwrap();
    }

    let cid = root.flush().unwrap();
    let s = cid.to_string();
    println!("{}", s);
    assert_eq!(
        &s,
        "bafy2bzacedtys7tutnbv7677lkpkrkzduhcgwybj4m4vl5pmdwujnsmnq5e6s"
    );

    let mut collection = vec![];
    let _ = root.for_each(&mut |key, value| {
        collection.push((key, value.clone()));
        Ok(())
    });
    m.sort();
    for (src, (key, value)) in m.iter().zip(collection.into_iter()) {
        assert_eq!(*src, key);
        assert_eq!(Value::Integer(*src as i128), value);
    }

    for (src, (key, _)) in m.iter().zip(root.iter()) {
        assert_eq!(*src, key)
    }
}

fn assert_get<B: Blocks>(root: &Amt<B>, key: u64, value: &str) {
    let s: String = root.get(key).unwrap();
    assert_eq!(&s, value);
}

#[test]
fn amt_basic_get_set_test() {
    let bs = db();
    let mut root = Amt::new(bs.clone());
    root.set(2, "foo").unwrap();
    assert_get(&root, 2, "foo");
    assert_eq!(root.count(), 1);

    let c = root.flush().unwrap();

    let root = Amt::load(&c, bs).unwrap();
    assert_get(&root, 2, "foo");
    assert_eq!(root.count(), 1);
}

#[test]
fn test_expand() {
    let bs = db();
    let mut root = Amt::new(bs.clone());
    root.set(2, "foo").unwrap();
    root.set(11, "bar").unwrap();
    root.set(79, "baz").unwrap();

    assert_get(&root, 2, "foo");
    assert_get(&root, 11, "bar");
    assert_get(&root, 79, "baz");

    let c = root.flush().unwrap();
    let root = Amt::load(&c, bs).unwrap();

    assert_get(&root, 2, "foo");
    assert_get(&root, 11, "bar");
    assert_get(&root, 79, "baz");
}

#[test]
fn test_insert_a_bunch() {
    let bs = db();
    let mut root = Amt::new(bs.clone());
    let num = 5000;

    for i in 0..num {
        root.set(i, "foo foo bar").unwrap();
    }

    for i in 0..num {
        assert_get(&root, i, "foo foo bar");
    }

    let c = root.flush().unwrap();
    assert_eq!(
        &c.to_string(),
        "bafy2bzacedjhcq7542wu7ike4i4srgq7hwxxc5pmw5sub4secqk33mugl4zda"
    );
    let root = Amt::load(&c, bs).unwrap();
    for i in 0..num {
        assert_get(&root, i, "foo foo bar");
    }
    assert_eq!(root.count(), num);
}

fn assert_delete<B: Blocks>(root: &mut Amt<B>, k: u64) {
    root.delete(k).unwrap();
    assert!(matches!(
        root.get::<String>(k),
        Err(AmtIpldError::NotFound(_k))
    ));
}

#[test]
fn test_delete_first_entry() {
    let bs = db();
    let mut root = Amt::new(bs.clone());

    root.set(0, "cat").unwrap();
    root.set(27, "cat").unwrap();

    assert_delete(&mut root, 0);

    let c = root.flush().unwrap();

    let root = Amt::load(&c, bs).unwrap();
    assert_eq!(root.count(), 1);
}

#[test]
fn test_delete() {
    let bs = db();
    let mut root = Amt::new(bs.clone());

    root.set(0, "cat").unwrap();
    root.set(1, "cat").unwrap();
    root.set(2, "cat").unwrap();
    root.set(3, "cat").unwrap();

    assert_delete(&mut root, 1);

    assert_get(&root, 0, "cat");
    assert_get(&root, 2, "cat");
    assert_get(&root, 3, "cat");

    assert_delete(&mut root, 0);
    assert_delete(&mut root, 2);
    assert_delete(&mut root, 3);

    assert_eq!(root.count(), 0);

    root.set(23, "dog").unwrap();
    root.set(24, "dog").unwrap();

    assert_delete(&mut root, 23);

    assert_eq!(root.count(), 1);

    let c = root.flush().unwrap();
    let new_root = Amt::load(&c, bs.clone()).unwrap();
    assert_eq!(new_root.count(), 1);

    let mut root2 = Amt::new(bs);
    root2.set(24, "dog").unwrap();
    let c2 = root.flush().unwrap();
    assert_eq!(c, c2);
}

#[test]
fn test_delete_reduce_height() {
    let bs = db();
    let mut root = Amt::new(bs.clone());

    root.set(1, "thing").unwrap();
    let c1 = root.flush().unwrap();

    root.set(37, "other").unwrap();

    let c2 = root.flush().unwrap();

    let mut root2 = Amt::load(&c2, bs).unwrap();

    assert_delete(&mut root2, 37);
    assert_eq!(root2.count(), 1);

    let c3 = root2.flush().unwrap();

    assert_eq!(c1, c3);
}

#[test]
fn test_for_each() {
    let bs = db();
    let mut root = Amt::new(bs.clone());

    for i in INDEXS.iter() {
        root.set(*i, "value").unwrap();
    }
    let c1 = root.flush().unwrap();

    println!("c1:{}", c1.to_string());
    assert_eq!(
        &c1.to_string(),
        "bafy2bzacectf4h75apastjwzazkksxcvbjwjxpw5g2xwsety5oichvziywdew"
    );

    for i in INDEXS.iter() {
        assert_get(&root, *i, "value");
    }

    assert_eq!(root.count(), INDEXS.len() as u64);

    let c2 = root.flush().unwrap();
    println!("c2:{}", c2.to_string());

    assert_eq!(
        &c2.to_string(),
        "bafy2bzacectf4h75apastjwzazkksxcvbjwjxpw5g2xwsety5oichvziywdew"
    );
    assert_eq!(c1.to_string(), c2.to_string());

    let root2 = Amt::load(&c2, bs).unwrap();
    assert_eq!(root2.count(), INDEXS.len() as u64);

    let mut x = 0;
    let _ = root2.for_each(&mut |key, _| {
        assert_eq!(key, INDEXS[x]);
        x += 1;
        Ok(())
    });

    assert_eq!(x, INDEXS.len());

    x = 0;
    for (_, item) in root2.iter().enumerate() {
        assert_eq!(item.0, INDEXS[x]);
        x += 1;
    }
    assert_eq!(x, INDEXS.len());
}

const INDEXS: [u64; 4933] = [
    1, 3, 8, 11, 14, 15, 17, 20, 23, 25, 29, 31, 32, 33, 34, 36, 37, 38, 40, 41, 42, 43, 47, 48,
    49, 51, 53, 55, 56, 60, 62, 64, 69, 72, 73, 74, 77, 79, 80, 84, 85, 88, 90, 91, 92, 94, 95, 96,
    97, 103, 104, 105, 106, 108, 109, 110, 113, 117, 119, 120, 121, 125, 128, 129, 136, 139, 140,
    142, 143, 144, 146, 148, 149, 152, 154, 155, 159, 162, 163, 165, 166, 167, 168, 169, 170, 173,
    174, 175, 177, 178, 182, 183, 185, 186, 192, 194, 195, 196, 197, 198, 201, 202, 203, 205, 206,
    207, 210, 211, 213, 214, 216, 218, 219, 220, 221, 222, 223, 224, 227, 228, 229, 230, 231, 233,
    234, 236, 237, 239, 241, 243, 246, 249, 252, 253, 254, 255, 256, 257, 259, 261, 262, 264, 265,
    270, 276, 277, 278, 279, 280, 282, 283, 284, 285, 286, 288, 289, 290, 291, 293, 294, 295, 296,
    300, 302, 305, 306, 307, 308, 310, 311, 312, 314, 317, 318, 321, 323, 324, 326, 328, 332, 333,
    334, 335, 338, 339, 340, 341, 343, 344, 346, 347, 350, 351, 353, 359, 363, 364, 366, 367, 370,
    371, 374, 375, 378, 380, 381, 383, 384, 386, 388, 389, 391, 393, 395, 402, 403, 404, 405, 408,
    409, 410, 411, 413, 416, 417, 426, 427, 432, 434, 436, 440, 441, 443, 444, 451, 454, 456, 458,
    459, 463, 464, 469, 470, 472, 474, 477, 478, 479, 485, 486, 487, 489, 490, 493, 499, 500, 501,
    502, 503, 506, 507, 510, 513, 514, 516, 518, 519, 520, 524, 529, 530, 536, 537, 539, 541, 543,
    544, 545, 549, 550, 552, 553, 556, 557, 558, 559, 560, 561, 567, 568, 569, 570, 583, 584, 585,
    586, 587, 588, 590, 591, 599, 600, 602, 603, 606, 607, 609, 612, 614, 618, 621, 622, 624, 627,
    629, 632, 633, 634, 636, 637, 640, 641, 642, 644, 645, 646, 648, 651, 652, 655, 656, 657, 658,
    661, 665, 669, 670, 673, 676, 679, 682, 685, 687, 690, 692, 693, 695, 697, 699, 705, 708, 712,
    714, 715, 721, 722, 723, 726, 727, 728, 731, 734, 736, 737, 738, 739, 740, 741, 743, 746, 751,
    752, 755, 758, 761, 764, 768, 769, 770, 771, 774, 777, 779, 780, 783, 785, 787, 789, 790, 791,
    792, 793, 795, 797, 799, 801, 808, 810, 811, 812, 814, 815, 816, 819, 823, 824, 827, 829, 831,
    834, 835, 837, 839, 840, 841, 842, 843, 844, 846, 847, 849, 854, 855, 856, 858, 860, 862, 863,
    865, 866, 868, 869, 873, 877, 878, 881, 882, 883, 885, 886, 887, 888, 889, 890, 891, 892, 894,
    895, 896, 899, 900, 903, 904, 905, 906, 908, 909, 914, 915, 916, 918, 925, 927, 928, 929, 930,
    931, 934, 937, 938, 939, 941, 948, 949, 950, 951, 952, 955, 959, 960, 962, 969, 971, 972, 973,
    974, 976, 980, 982, 984, 985, 986, 987, 997, 999, 1000, 1003, 1005, 1009, 1010, 1013, 1015,
    1018, 1019, 1020, 1024, 1025, 1027, 1028, 1030, 1031, 1032, 1033, 1034, 1035, 1037, 1038, 1040,
    1041, 1043, 1044, 1045, 1046, 1049, 1050, 1051, 1053, 1058, 1062, 1063, 1067, 1070, 1071, 1073,
    1075, 1080, 1085, 1086, 1088, 1089, 1090, 1094, 1095, 1097, 1098, 1100, 1104, 1107, 1108, 1111,
    1112, 1114, 1116, 1118, 1121, 1122, 1123, 1125, 1126, 1127, 1129, 1131, 1133, 1134, 1137, 1138,
    1139, 1140, 1142, 1145, 1149, 1150, 1151, 1156, 1161, 1170, 1178, 1179, 1181, 1182, 1184, 1190,
    1192, 1193, 1194, 1196, 1197, 1200, 1201, 1203, 1205, 1206, 1211, 1213, 1215, 1216, 1221, 1223,
    1225, 1229, 1231, 1233, 1235, 1236, 1241, 1242, 1245, 1246, 1251, 1254, 1256, 1257, 1259, 1260,
    1261, 1263, 1267, 1269, 1273, 1276, 1277, 1278, 1279, 1280, 1281, 1282, 1284, 1286, 1294, 1297,
    1298, 1301, 1302, 1303, 1307, 1308, 1309, 1313, 1314, 1316, 1320, 1321, 1325, 1327, 1331, 1333,
    1334, 1335, 1339, 1340, 1341, 1342, 1343, 1344, 1345, 1346, 1347, 1350, 1351, 1352, 1355, 1357,
    1362, 1363, 1364, 1366, 1368, 1370, 1374, 1375, 1378, 1380, 1382, 1383, 1384, 1385, 1387, 1389,
    1391, 1393, 1394, 1396, 1398, 1401, 1404, 1406, 1408, 1409, 1414, 1415, 1417, 1419, 1421, 1423,
    1427, 1429, 1430, 1432, 1434, 1438, 1440, 1442, 1443, 1446, 1451, 1454, 1455, 1456, 1457, 1458,
    1461, 1462, 1463, 1469, 1470, 1471, 1472, 1473, 1474, 1476, 1477, 1478, 1484, 1487, 1488, 1489,
    1495, 1496, 1497, 1498, 1499, 1502, 1503, 1505, 1507, 1513, 1514, 1515, 1516, 1517, 1518, 1525,
    1526, 1527, 1528, 1529, 1531, 1533, 1536, 1537, 1539, 1540, 1542, 1544, 1546, 1548, 1549, 1550,
    1552, 1554, 1555, 1560, 1561, 1562, 1563, 1565, 1568, 1569, 1570, 1573, 1575, 1576, 1577, 1578,
    1579, 1580, 1581, 1583, 1587, 1590, 1591, 1592, 1593, 1595, 1596, 1598, 1599, 1600, 1602, 1603,
    1604, 1606, 1607, 1613, 1616, 1617, 1620, 1621, 1622, 1624, 1625, 1629, 1630, 1631, 1633, 1634,
    1638, 1639, 1640, 1648, 1649, 1650, 1651, 1653, 1654, 1655, 1657, 1658, 1660, 1661, 1663, 1665,
    1666, 1667, 1669, 1671, 1672, 1675, 1676, 1677, 1679, 1682, 1686, 1688, 1689, 1691, 1696, 1697,
    1698, 1699, 1700, 1702, 1703, 1704, 1706, 1708, 1710, 1713, 1714, 1716, 1719, 1722, 1723, 1726,
    1728, 1729, 1730, 1733, 1734, 1735, 1739, 1742, 1743, 1744, 1745, 1746, 1747, 1748, 1750, 1751,
    1752, 1755, 1756, 1757, 1758, 1762, 1763, 1767, 1768, 1769, 1774, 1776, 1777, 1778, 1780, 1782,
    1783, 1784, 1785, 1786, 1788, 1790, 1792, 1797, 1800, 1803, 1804, 1808, 1810, 1811, 1815, 1817,
    1822, 1823, 1824, 1826, 1829, 1833, 1834, 1841, 1842, 1843, 1844, 1845, 1846, 1849, 1850, 1853,
    1854, 1862, 1867, 1868, 1873, 1874, 1875, 1877, 1878, 1879, 1882, 1883, 1888, 1898, 1899, 1900,
    1903, 1906, 1907, 1908, 1909, 1912, 1913, 1915, 1916, 1920, 1921, 1922, 1924, 1926, 1928, 1929,
    1931, 1932, 1933, 1936, 1937, 1938, 1940, 1945, 1946, 1947, 1952, 1953, 1957, 1960, 1965, 1970,
    1972, 1973, 1977, 1978, 1980, 1982, 1983, 1984, 1987, 1988, 1989, 1993, 1994, 1995, 1999, 2000,
    2002, 2005, 2007, 2009, 2010, 2011, 2013, 2016, 2017, 2018, 2019, 2020, 2022, 2023, 2024, 2027,
    2028, 2029, 2031, 2032, 2034, 2036, 2038, 2040, 2045, 2046, 2049, 2050, 2052, 2053, 2055, 2056,
    2057, 2058, 2060, 2063, 2066, 2069, 2071, 2072, 2074, 2079, 2080, 2081, 2086, 2089, 2091, 2093,
    2094, 2095, 2097, 2098, 2099, 2101, 2103, 2105, 2106, 2110, 2111, 2113, 2114, 2115, 2116, 2117,
    2119, 2123, 2124, 2125, 2126, 2128, 2130, 2133, 2135, 2137, 2140, 2141, 2144, 2148, 2150, 2152,
    2154, 2156, 2157, 2161, 2165, 2166, 2167, 2168, 2169, 2170, 2171, 2172, 2173, 2174, 2175, 2176,
    2182, 2183, 2186, 2188, 2189, 2192, 2193, 2196, 2199, 2201, 2202, 2203, 2205, 2207, 2208, 2209,
    2212, 2214, 2216, 2220, 2223, 2224, 2226, 2227, 2228, 2230, 2235, 2236, 2237, 2239, 2240, 2241,
    2245, 2247, 2248, 2250, 2251, 2253, 2255, 2263, 2266, 2268, 2269, 2270, 2272, 2273, 2276, 2277,
    2278, 2280, 2286, 2288, 2294, 2295, 2297, 2299, 2300, 2301, 2303, 2304, 2306, 2307, 2310, 2312,
    2313, 2314, 2315, 2319, 2320, 2323, 2327, 2328, 2331, 2332, 2334, 2335, 2336, 2337, 2339, 2341,
    2342, 2343, 2345, 2346, 2347, 2348, 2352, 2355, 2356, 2358, 2360, 2361, 2363, 2364, 2365, 2368,
    2369, 2370, 2372, 2373, 2376, 2378, 2380, 2383, 2384, 2388, 2389, 2390, 2391, 2392, 2395, 2397,
    2400, 2401, 2403, 2407, 2408, 2410, 2411, 2412, 2413, 2414, 2415, 2417, 2419, 2420, 2424, 2425,
    2427, 2428, 2431, 2432, 2433, 2434, 2435, 2437, 2440, 2441, 2444, 2448, 2450, 2455, 2458, 2459,
    2460, 2461, 2463, 2464, 2469, 2470, 2471, 2474, 2477, 2478, 2479, 2480, 2488, 2497, 2501, 2503,
    2505, 2507, 2508, 2510, 2511, 2512, 2513, 2514, 2516, 2518, 2519, 2520, 2523, 2530, 2531, 2532,
    2535, 2536, 2540, 2543, 2546, 2548, 2550, 2551, 2552, 2553, 2555, 2557, 2559, 2561, 2564, 2566,
    2567, 2568, 2569, 2572, 2574, 2577, 2578, 2579, 2581, 2582, 2583, 2586, 2590, 2591, 2593, 2594,
    2595, 2597, 2603, 2604, 2605, 2611, 2612, 2616, 2617, 2618, 2619, 2620, 2623, 2625, 2626, 2627,
    2629, 2630, 2631, 2632, 2635, 2636, 2639, 2649, 2650, 2653, 2656, 2657, 2660, 2666, 2670, 2673,
    2676, 2677, 2678, 2679, 2680, 2681, 2682, 2683, 2684, 2686, 2687, 2689, 2690, 2692, 2694, 2698,
    2699, 2701, 2702, 2703, 2704, 2706, 2709, 2712, 2713, 2715, 2717, 2718, 2720, 2723, 2726, 2727,
    2729, 2730, 2731, 2732, 2733, 2734, 2736, 2738, 2740, 2743, 2746, 2747, 2750, 2751, 2752, 2754,
    2756, 2759, 2760, 2762, 2763, 2764, 2765, 2766, 2767, 2768, 2771, 2776, 2778, 2779, 2780, 2781,
    2782, 2784, 2785, 2786, 2787, 2789, 2793, 2795, 2796, 2797, 2798, 2803, 2805, 2807, 2808, 2811,
    2815, 2818, 2825, 2829, 2834, 2835, 2836, 2837, 2839, 2840, 2841, 2842, 2846, 2847, 2848, 2849,
    2850, 2851, 2854, 2857, 2859, 2861, 2868, 2869, 2871, 2872, 2873, 2874, 2875, 2877, 2879, 2885,
    2886, 2890, 2891, 2893, 2898, 2899, 2901, 2905, 2909, 2915, 2916, 2918, 2919, 2920, 2921, 2926,
    2928, 2929, 2933, 2934, 2936, 2937, 2938, 2941, 2942, 2945, 2950, 2952, 2953, 2956, 2957, 2960,
    2962, 2965, 2966, 2967, 2968, 2970, 2971, 2974, 2975, 2976, 2981, 2982, 2983, 2985, 2986, 2988,
    2990, 2993, 2994, 2997, 2998, 2999, 3001, 3003, 3006, 3007, 3008, 3010, 3011, 3012, 3013, 3016,
    3017, 3018, 3019, 3025, 3027, 3028, 3029, 3030, 3032, 3034, 3035, 3038, 3040, 3041, 3047, 3048,
    3051, 3052, 3053, 3054, 3055, 3056, 3057, 3059, 3061, 3062, 3063, 3065, 3067, 3072, 3073, 3075,
    3077, 3080, 3081, 3082, 3085, 3086, 3087, 3088, 3089, 3091, 3092, 3094, 3096, 3097, 3098, 3099,
    3101, 3103, 3104, 3109, 3110, 3111, 3112, 3114, 3117, 3118, 3122, 3123, 3130, 3132, 3133, 3134,
    3144, 3145, 3147, 3150, 3152, 3153, 3154, 3155, 3158, 3161, 3167, 3171, 3173, 3175, 3177, 3179,
    3180, 3184, 3185, 3186, 3187, 3190, 3194, 3196, 3199, 3202, 3203, 3205, 3208, 3209, 3211, 3215,
    3217, 3220, 3224, 3225, 3230, 3235, 3236, 3238, 3239, 3240, 3243, 3244, 3245, 3249, 3250, 3251,
    3252, 3254, 3255, 3256, 3258, 3259, 3260, 3262, 3264, 3267, 3269, 3270, 3272, 3273, 3274, 3278,
    3281, 3282, 3285, 3286, 3288, 3290, 3291, 3292, 3293, 3294, 3296, 3297, 3302, 3303, 3305, 3306,
    3310, 3311, 3312, 3316, 3317, 3320, 3321, 3322, 3323, 3324, 3325, 3330, 3333, 3334, 3335, 3340,
    3344, 3346, 3348, 3349, 3351, 3359, 3361, 3367, 3371, 3372, 3381, 3382, 3383, 3385, 3386, 3388,
    3391, 3393, 3397, 3400, 3401, 3402, 3404, 3405, 3406, 3409, 3410, 3414, 3416, 3417, 3418, 3419,
    3423, 3429, 3430, 3431, 3434, 3437, 3438, 3440, 3441, 3444, 3445, 3446, 3448, 3449, 3450, 3452,
    3454, 3455, 3456, 3457, 3458, 3459, 3460, 3461, 3462, 3463, 3468, 3469, 3471, 3473, 3474, 3477,
    3480, 3482, 3483, 3489, 3490, 3491, 3494, 3495, 3496, 3497, 3500, 3501, 3502, 3503, 3505, 3508,
    3509, 3510, 3512, 3514, 3516, 3517, 3518, 3524, 3525, 3526, 3527, 3528, 3529, 3532, 3538, 3540,
    3545, 3546, 3548, 3549, 3553, 3555, 3556, 3558, 3559, 3560, 3563, 3564, 3565, 3568, 3569, 3570,
    3571, 3575, 3576, 3579, 3580, 3582, 3584, 3585, 3586, 3589, 3590, 3591, 3592, 3595, 3604, 3606,
    3607, 3611, 3612, 3614, 3615, 3617, 3618, 3619, 3621, 3622, 3627, 3631, 3632, 3635, 3636, 3637,
    3640, 3642, 3643, 3648, 3651, 3652, 3653, 3654, 3655, 3656, 3657, 3659, 3661, 3664, 3666, 3669,
    3671, 3672, 3674, 3678, 3680, 3681, 3685, 3686, 3687, 3688, 3689, 3690, 3694, 3695, 3696, 3697,
    3700, 3702, 3704, 3706, 3708, 3710, 3712, 3715, 3716, 3719, 3721, 3724, 3725, 3726, 3727, 3729,
    3731, 3733, 3737, 3738, 3739, 3740, 3741, 3742, 3743, 3744, 3748, 3749, 3750, 3751, 3755, 3759,
    3761, 3762, 3763, 3764, 3765, 3766, 3767, 3769, 3770, 3771, 3772, 3775, 3777, 3778, 3782, 3784,
    3785, 3786, 3789, 3790, 3791, 3792, 3794, 3796, 3797, 3798, 3799, 3801, 3803, 3806, 3807, 3808,
    3809, 3815, 3819, 3822, 3823, 3824, 3827, 3829, 3831, 3833, 3834, 3835, 3836, 3840, 3843, 3845,
    3846, 3847, 3849, 3850, 3851, 3852, 3855, 3858, 3859, 3861, 3862, 3864, 3865, 3866, 3867, 3868,
    3869, 3870, 3871, 3872, 3874, 3875, 3876, 3877, 3882, 3884, 3886, 3887, 3888, 3892, 3894, 3895,
    3897, 3899, 3900, 3901, 3902, 3909, 3911, 3912, 3914, 3916, 3919, 3922, 3925, 3926, 3928, 3929,
    3931, 3932, 3934, 3938, 3939, 3941, 3942, 3946, 3949, 3951, 3952, 3953, 3956, 3957, 3962, 3963,
    3965, 3966, 3967, 3968, 3969, 3970, 3971, 3973, 3976, 3977, 3989, 3991, 3997, 3998, 3999, 4000,
    4001, 4002, 4003, 4004, 4008, 4011, 4012, 4014, 4016, 4017, 4018, 4020, 4021, 4022, 4023, 4024,
    4025, 4028, 4030, 4031, 4038, 4039, 4040, 4042, 4044, 4046, 4047, 4050, 4052, 4054, 4055, 4057,
    4058, 4059, 4060, 4063, 4066, 4067, 4068, 4074, 4077, 4078, 4081, 4084, 4089, 4091, 4096, 4098,
    4100, 4101, 4102, 4103, 4105, 4111, 4113, 4116, 4118, 4120, 4121, 4123, 4130, 4134, 4138, 4139,
    4140, 4143, 4145, 4146, 4147, 4148, 4149, 4152, 4155, 4159, 4161, 4163, 4165, 4167, 4169, 4171,
    4172, 4173, 4175, 4176, 4184, 4185, 4187, 4189, 4190, 4191, 4192, 4193, 4195, 4196, 4201, 4203,
    4205, 4206, 4208, 4209, 4212, 4213, 4215, 4216, 4218, 4221, 4222, 4223, 4227, 4229, 4230, 4231,
    4232, 4234, 4235, 4237, 4239, 4240, 4242, 4243, 4246, 4247, 4250, 4252, 4253, 4257, 4258, 4259,
    4260, 4263, 4265, 4268, 4270, 4274, 4276, 4277, 4279, 4283, 4284, 4286, 4287, 4288, 4289, 4290,
    4298, 4301, 4303, 4304, 4305, 4307, 4313, 4314, 4318, 4320, 4322, 4323, 4324, 4327, 4328, 4330,
    4331, 4334, 4335, 4337, 4339, 4340, 4342, 4343, 4344, 4345, 4350, 4351, 4352, 4354, 4355, 4356,
    4357, 4360, 4362, 4363, 4364, 4365, 4370, 4371, 4372, 4373, 4374, 4375, 4378, 4382, 4384, 4387,
    4389, 4391, 4393, 4395, 4398, 4400, 4401, 4402, 4404, 4407, 4408, 4409, 4410, 4413, 4414, 4418,
    4419, 4420, 4422, 4424, 4427, 4428, 4429, 4430, 4431, 4434, 4436, 4437, 4439, 4440, 4441, 4445,
    4447, 4448, 4450, 4455, 4457, 4459, 4460, 4463, 4466, 4467, 4468, 4472, 4475, 4479, 4480, 4487,
    4488, 4489, 4490, 4491, 4492, 4494, 4495, 4498, 4500, 4503, 4505, 4506, 4508, 4509, 4510, 4511,
    4517, 4519, 4521, 4522, 4524, 4525, 4532, 4533, 4535, 4536, 4537, 4538, 4539, 4540, 4541, 4542,
    4545, 4548, 4552, 4555, 4563, 4564, 4567, 4569, 4571, 4574, 4575, 4583, 4586, 4587, 4589, 4592,
    4594, 4597, 4598, 4602, 4606, 4611, 4613, 4616, 4617, 4619, 4620, 4621, 4625, 4626, 4628, 4629,
    4631, 4632, 4633, 4634, 4635, 4637, 4639, 4640, 4644, 4647, 4653, 4656, 4659, 4660, 4661, 4662,
    4664, 4665, 4667, 4668, 4669, 4670, 4672, 4673, 4674, 4676, 4678, 4680, 4682, 4684, 4685, 4687,
    4688, 4691, 4698, 4701, 4702, 4703, 4706, 4707, 4710, 4711, 4713, 4714, 4716, 4717, 4719, 4720,
    4723, 4725, 4726, 4730, 4731, 4733, 4734, 4735, 4737, 4738, 4742, 4743, 4745, 4748, 4749, 4750,
    4751, 4756, 4758, 4760, 4764, 4765, 4766, 4770, 4772, 4773, 4774, 4775, 4776, 4777, 4778, 4781,
    4783, 4784, 4787, 4789, 4794, 4795, 4796, 4799, 4801, 4802, 4806, 4807, 4809, 4813, 4816, 4817,
    4820, 4823, 4824, 4825, 4827, 4829, 4830, 4832, 4833, 4834, 4837, 4838, 4840, 4842, 4844, 4847,
    4848, 4850, 4851, 4852, 4855, 4857, 4859, 4862, 4863, 4867, 4869, 4870, 4871, 4875, 4876, 4879,
    4880, 4884, 4885, 4886, 4888, 4889, 4891, 4894, 4895, 4898, 4899, 4900, 4902, 4905, 4907, 4913,
    4914, 4915, 4916, 4919, 4921, 4922, 4923, 4928, 4929, 4932, 4933, 4934, 4936, 4937, 4939, 4940,
    4941, 4943, 4944, 4945, 4948, 4952, 4953, 4954, 4958, 4959, 4970, 4975, 4976, 4978, 4984, 4985,
    4986, 4988, 4989, 4991, 4992, 4993, 4994, 4995, 4997, 4998, 4999, 5001, 5003, 5004, 5008, 5009,
    5010, 5011, 5012, 5013, 5015, 5018, 5020, 5022, 5024, 5027, 5029, 5031, 5034, 5036, 5037, 5038,
    5039, 5040, 5042, 5045, 5049, 5050, 5054, 5057, 5061, 5065, 5070, 5071, 5073, 5074, 5075, 5076,
    5077, 5078, 5079, 5081, 5082, 5083, 5085, 5087, 5088, 5090, 5092, 5094, 5097, 5099, 5105, 5106,
    5109, 5112, 5113, 5114, 5117, 5120, 5124, 5128, 5131, 5133, 5134, 5136, 5141, 5142, 5143, 5145,
    5150, 5151, 5152, 5153, 5157, 5159, 5163, 5164, 5166, 5167, 5170, 5171, 5172, 5174, 5178, 5180,
    5183, 5184, 5185, 5189, 5192, 5194, 5197, 5198, 5199, 5200, 5201, 5203, 5205, 5209, 5210, 5211,
    5212, 5215, 5218, 5219, 5221, 5222, 5223, 5231, 5233, 5234, 5235, 5240, 5241, 5242, 5243, 5245,
    5246, 5247, 5253, 5254, 5258, 5260, 5262, 5263, 5265, 5266, 5267, 5268, 5269, 5272, 5276, 5279,
    5280, 5282, 5283, 5286, 5287, 5288, 5290, 5293, 5295, 5296, 5298, 5301, 5302, 5303, 5307, 5308,
    5309, 5312, 5314, 5316, 5317, 5318, 5319, 5323, 5324, 5325, 5329, 5330, 5331, 5333, 5335, 5336,
    5337, 5346, 5347, 5354, 5361, 5362, 5363, 5367, 5369, 5370, 5372, 5373, 5375, 5378, 5379, 5381,
    5385, 5387, 5392, 5395, 5396, 5397, 5400, 5402, 5403, 5405, 5406, 5407, 5408, 5409, 5411, 5414,
    5415, 5417, 5420, 5421, 5422, 5423, 5425, 5426, 5429, 5433, 5435, 5436, 5437, 5441, 5445, 5448,
    5449, 5450, 5452, 5454, 5456, 5457, 5458, 5460, 5461, 5463, 5466, 5467, 5469, 5471, 5478, 5479,
    5480, 5481, 5487, 5488, 5492, 5493, 5494, 5495, 5496, 5497, 5498, 5501, 5505, 5508, 5510, 5513,
    5514, 5518, 5520, 5521, 5523, 5524, 5525, 5527, 5530, 5532, 5534, 5535, 5536, 5537, 5539, 5540,
    5541, 5547, 5548, 5552, 5554, 5556, 5559, 5561, 5564, 5565, 5566, 5570, 5571, 5572, 5574, 5576,
    5578, 5582, 5585, 5589, 5590, 5592, 5595, 5598, 5599, 5603, 5606, 5609, 5610, 5611, 5612, 5614,
    5615, 5616, 5617, 5618, 5619, 5622, 5623, 5625, 5626, 5627, 5628, 5630, 5631, 5632, 5633, 5634,
    5637, 5640, 5642, 5643, 5645, 5646, 5650, 5651, 5653, 5655, 5656, 5657, 5658, 5661, 5665, 5667,
    5671, 5672, 5679, 5680, 5683, 5685, 5686, 5687, 5688, 5689, 5694, 5696, 5698, 5702, 5703, 5704,
    5705, 5711, 5712, 5715, 5716, 5717, 5722, 5723, 5724, 5725, 5728, 5730, 5733, 5735, 5736, 5739,
    5743, 5744, 5745, 5747, 5750, 5751, 5752, 5753, 5755, 5756, 5758, 5759, 5763, 5764, 5765, 5768,
    5771, 5776, 5777, 5780, 5781, 5782, 5783, 5784, 5785, 5786, 5787, 5789, 5793, 5794, 5795, 5796,
    5797, 5798, 5799, 5800, 5801, 5803, 5804, 5806, 5809, 5814, 5815, 5816, 5817, 5821, 5822, 5827,
    5829, 5830, 5832, 5833, 5834, 5836, 5837, 5838, 5839, 5841, 5843, 5846, 5847, 5851, 5852, 5854,
    5855, 5856, 5859, 5860, 5861, 5865, 5867, 5870, 5876, 5877, 5878, 5879, 5880, 5881, 5885, 5886,
    5887, 5889, 5890, 5892, 5894, 5895, 5896, 5899, 5900, 5902, 5903, 5904, 5907, 5908, 5910, 5914,
    5915, 5917, 5918, 5919, 5921, 5924, 5925, 5926, 5928, 5929, 5930, 5931, 5935, 5937, 5938, 5939,
    5940, 5942, 5945, 5946, 5953, 5955, 5958, 5964, 5966, 5970, 5972, 5973, 5977, 5981, 5982, 5983,
    5984, 5985, 5986, 5988, 5990, 5992, 5998, 5999, 6002, 6003, 6005, 6006, 6007, 6008, 6012, 6015,
    6016, 6020, 6021, 6022, 6025, 6030, 6034, 6035, 6038, 6039, 6040, 6042, 6043, 6044, 6045, 6047,
    6051, 6052, 6053, 6058, 6059, 6060, 6061, 6062, 6064, 6065, 6071, 6073, 6074, 6075, 6076, 6077,
    6078, 6080, 6081, 6083, 6085, 6089, 6093, 6094, 6095, 6096, 6097, 6098, 6099, 6101, 6104, 6105,
    6106, 6108, 6111, 6112, 6116, 6119, 6121, 6122, 6125, 6131, 6134, 6138, 6140, 6141, 6143, 6144,
    6145, 6146, 6147, 6148, 6149, 6150, 6151, 6152, 6153, 6154, 6155, 6158, 6159, 6160, 6161, 6165,
    6171, 6172, 6174, 6176, 6178, 6181, 6184, 6186, 6187, 6188, 6191, 6193, 6194, 6197, 6201, 6202,
    6203, 6204, 6205, 6206, 6207, 6208, 6212, 6214, 6218, 6220, 6221, 6223, 6228, 6229, 6230, 6233,
    6236, 6240, 6241, 6244, 6249, 6252, 6254, 6256, 6257, 6258, 6261, 6262, 6266, 6267, 6268, 6269,
    6270, 6271, 6272, 6273, 6274, 6276, 6277, 6279, 6282, 6283, 6285, 6293, 6295, 6296, 6297, 6301,
    6302, 6303, 6310, 6314, 6316, 6317, 6320, 6322, 6323, 6325, 6326, 6328, 6330, 6331, 6335, 6338,
    6341, 6343, 6347, 6348, 6349, 6350, 6351, 6352, 6353, 6356, 6358, 6359, 6361, 6362, 6363, 6365,
    6366, 6371, 6373, 6378, 6384, 6387, 6388, 6390, 6392, 6395, 6396, 6398, 6402, 6403, 6408, 6411,
    6413, 6418, 6424, 6425, 6426, 6428, 6430, 6432, 6439, 6440, 6442, 6443, 6444, 6447, 6448, 6456,
    6457, 6458, 6459, 6461, 6463, 6467, 6471, 6472, 6473, 6474, 6475, 6477, 6478, 6480, 6481, 6482,
    6483, 6484, 6485, 6492, 6497, 6499, 6501, 6502, 6503, 6505, 6506, 6507, 6508, 6509, 6510, 6511,
    6514, 6516, 6518, 6519, 6523, 6527, 6528, 6531, 6533, 6534, 6537, 6539, 6543, 6544, 6545, 6547,
    6548, 6549, 6550, 6551, 6552, 6553, 6556, 6560, 6562, 6564, 6565, 6566, 6567, 6568, 6571, 6572,
    6573, 6575, 6576, 6580, 6585, 6586, 6588, 6589, 6593, 6594, 6596, 6597, 6598, 6603, 6604, 6606,
    6608, 6613, 6615, 6622, 6623, 6628, 6629, 6632, 6634, 6635, 6638, 6641, 6642, 6644, 6648, 6650,
    6651, 6652, 6653, 6654, 6655, 6657, 6658, 6660, 6661, 6665, 6666, 6667, 6668, 6669, 6674, 6677,
    6680, 6687, 6690, 6694, 6695, 6697, 6700, 6704, 6705, 6707, 6712, 6713, 6714, 6715, 6716, 6720,
    6723, 6725, 6731, 6733, 6734, 6738, 6739, 6740, 6741, 6745, 6746, 6747, 6752, 6753, 6754, 6756,
    6757, 6759, 6761, 6762, 6763, 6765, 6772, 6774, 6775, 6776, 6782, 6783, 6789, 6790, 6792, 6796,
    6800, 6801, 6805, 6808, 6809, 6810, 6811, 6814, 6815, 6817, 6818, 6822, 6823, 6824, 6825, 6826,
    6827, 6828, 6831, 6834, 6837, 6838, 6841, 6842, 6843, 6844, 6845, 6846, 6847, 6848, 6850, 6853,
    6854, 6855, 6856, 6858, 6862, 6864, 6865, 6866, 6867, 6868, 6870, 6872, 6873, 6874, 6877, 6879,
    6881, 6883, 6884, 6886, 6887, 6889, 6890, 6891, 6892, 6896, 6900, 6904, 6905, 6906, 6907, 6908,
    6909, 6910, 6913, 6916, 6917, 6918, 6919, 6920, 6922, 6923, 6927, 6928, 6929, 6930, 6931, 6937,
    6939, 6940, 6941, 6943, 6944, 6945, 6946, 6947, 6948, 6950, 6955, 6956, 6958, 6959, 6963, 6964,
    6967, 6969, 6970, 6971, 6972, 6977, 6982, 6983, 6985, 6987, 6988, 6996, 6997, 6998, 6999, 7008,
    7012, 7013, 7016, 7020, 7022, 7025, 7026, 7028, 7030, 7031, 7034, 7035, 7040, 7041, 7047, 7049,
    7052, 7054, 7055, 7056, 7064, 7066, 7068, 7069, 7072, 7075, 7076, 7079, 7080, 7081, 7083, 7084,
    7085, 7087, 7088, 7089, 7090, 7091, 7092, 7095, 7096, 7098, 7099, 7109, 7112, 7113, 7114, 7116,
    7117, 7118, 7119, 7122, 7124, 7125, 7127, 7128, 7131, 7132, 7133, 7135, 7138, 7141, 7142, 7143,
    7144, 7145, 7146, 7147, 7151, 7152, 7153, 7155, 7159, 7163, 7164, 7167, 7169, 7170, 7173, 7174,
    7176, 7181, 7182, 7183, 7186, 7187, 7188, 7189, 7191, 7194, 7196, 7197, 7200, 7203, 7206, 7207,
    7209, 7210, 7213, 7216, 7217, 7219, 7220, 7223, 7225, 7228, 7229, 7230, 7231, 7232, 7233, 7237,
    7239, 7241, 7242, 7244, 7245, 7247, 7252, 7254, 7260, 7261, 7262, 7267, 7271, 7276, 7277, 7278,
    7280, 7281, 7282, 7286, 7288, 7289, 7291, 7293, 7295, 7296, 7299, 7302, 7304, 7307, 7310, 7312,
    7313, 7316, 7317, 7318, 7320, 7321, 7323, 7325, 7326, 7329, 7333, 7334, 7336, 7337, 7340, 7341,
    7345, 7346, 7350, 7352, 7355, 7357, 7362, 7363, 7366, 7367, 7368, 7369, 7372, 7374, 7375, 7377,
    7380, 7381, 7382, 7384, 7387, 7388, 7389, 7391, 7392, 7394, 7395, 7398, 7401, 7406, 7409, 7411,
    7412, 7414, 7418, 7419, 7420, 7424, 7426, 7428, 7430, 7435, 7436, 7438, 7443, 7444, 7446, 7450,
    7452, 7453, 7454, 7457, 7458, 7460, 7462, 7464, 7466, 7469, 7470, 7473, 7474, 7482, 7483, 7484,
    7486, 7487, 7488, 7489, 7490, 7491, 7493, 7496, 7501, 7502, 7503, 7506, 7507, 7508, 7509, 7513,
    7514, 7518, 7519, 7522, 7523, 7528, 7529, 7533, 7534, 7535, 7537, 7543, 7544, 7545, 7546, 7551,
    7553, 7555, 7556, 7557, 7560, 7562, 7567, 7568, 7570, 7571, 7572, 7574, 7576, 7581, 7582, 7584,
    7587, 7588, 7589, 7590, 7592, 7594, 7595, 7597, 7599, 7600, 7603, 7605, 7608, 7610, 7613, 7614,
    7616, 7617, 7619, 7621, 7623, 7625, 7626, 7627, 7629, 7631, 7636, 7640, 7642, 7645, 7646, 7649,
    7651, 7654, 7655, 7656, 7657, 7659, 7665, 7666, 7667, 7669, 7670, 7671, 7673, 7675, 7679, 7681,
    7685, 7689, 7692, 7696, 7699, 7703, 7705, 7706, 7707, 7712, 7719, 7721, 7722, 7723, 7724, 7726,
    7727, 7728, 7729, 7732, 7739, 7740, 7741, 7743, 7745, 7746, 7748, 7749, 7750, 7751, 7753, 7754,
    7757, 7758, 7761, 7762, 7765, 7768, 7769, 7770, 7773, 7774, 7777, 7778, 7780, 7781, 7782, 7783,
    7787, 7789, 7796, 7798, 7800, 7803, 7805, 7809, 7810, 7812, 7813, 7814, 7815, 7818, 7820, 7821,
    7826, 7828, 7829, 7830, 7831, 7832, 7835, 7836, 7837, 7838, 7842, 7851, 7859, 7870, 7873, 7874,
    7878, 7879, 7881, 7883, 7884, 7885, 7887, 7888, 7892, 7893, 7894, 7896, 7897, 7898, 7899, 7903,
    7904, 7906, 7907, 7909, 7911, 7915, 7916, 7917, 7920, 7924, 7925, 7927, 7929, 7930, 7931, 7932,
    7933, 7934, 7939, 7940, 7941, 7943, 7944, 7947, 7948, 7950, 7951, 7953, 7955, 7959, 7960, 7962,
    7966, 7968, 7971, 7973, 7977, 7978, 7979, 7982, 7983, 7984, 7987, 7990, 7992, 7994, 7999, 8001,
    8002, 8005, 8009, 8012, 8013, 8014, 8015, 8016, 8017, 8018, 8020, 8025, 8026, 8027, 8028, 8029,
    8031, 8032, 8033, 8034, 8036, 8037, 8038, 8042, 8043, 8045, 8046, 8050, 8054, 8060, 8061, 8065,
    8066, 8071, 8073, 8074, 8075, 8078, 8080, 8082, 8083, 8084, 8085, 8088, 8091, 8093, 8095, 8099,
    8100, 8101, 8103, 8104, 8105, 8107, 8108, 8110, 8112, 8116, 8118, 8120, 8121, 8122, 8125, 8127,
    8129, 8132, 8134, 8136, 8137, 8138, 8139, 8141, 8142, 8147, 8150, 8151, 8154, 8155, 8162, 8163,
    8165, 8169, 8170, 8171, 8172, 8173, 8174, 8177, 8178, 8179, 8182, 8183, 8185, 8189, 8195, 8196,
    8198, 8200, 8201, 8202, 8205, 8206, 8208, 8212, 8213, 8215, 8216, 8217, 8218, 8220, 8222, 8229,
    8230, 8231, 8232, 8234, 8235, 8238, 8239, 8242, 8243, 8244, 8246, 8247, 8248, 8250, 8252, 8253,
    8256, 8258, 8259, 8261, 8262, 8264, 8267, 8268, 8269, 8271, 8273, 8275, 8276, 8282, 8285, 8287,
    8288, 8289, 8290, 8291, 8293, 8294, 8297, 8299, 8301, 8303, 8305, 8306, 8307, 8308, 8309, 8310,
    8311, 8312, 8314, 8317, 8322, 8324, 8326, 8328, 8329, 8332, 8333, 8334, 8336, 8338, 8341, 8342,
    8344, 8349, 8357, 8358, 8359, 8361, 8363, 8364, 8366, 8368, 8369, 8373, 8375, 8377, 8378, 8379,
    8381, 8382, 8384, 8385, 8387, 8388, 8389, 8392, 8394, 8395, 8401, 8403, 8405, 8407, 8408, 8409,
    8410, 8411, 8415, 8416, 8417, 8418, 8419, 8420, 8422, 8427, 8430, 8431, 8432, 8433, 8435, 8437,
    8438, 8439, 8441, 8443, 8444, 8445, 8448, 8449, 8450, 8455, 8460, 8461, 8462, 8463, 8465, 8466,
    8467, 8473, 8479, 8484, 8485, 8486, 8488, 8491, 8494, 8495, 8498, 8502, 8503, 8509, 8511, 8517,
    8518, 8521, 8522, 8523, 8524, 8525, 8528, 8529, 8533, 8537, 8539, 8540, 8545, 8546, 8547, 8548,
    8549, 8550, 8552, 8556, 8557, 8560, 8561, 8562, 8563, 8565, 8566, 8567, 8569, 8572, 8573, 8575,
    8579, 8580, 8581, 8585, 8586, 8587, 8591, 8593, 8595, 8599, 8601, 8605, 8607, 8608, 8609, 8610,
    8611, 8614, 8615, 8617, 8619, 8620, 8623, 8624, 8625, 8626, 8628, 8629, 8630, 8632, 8633, 8635,
    8637, 8638, 8639, 8641, 8642, 8643, 8648, 8652, 8654, 8655, 8661, 8667, 8668, 8672, 8673, 8674,
    8677, 8679, 8683, 8684, 8686, 8687, 8689, 8690, 8692, 8693, 8699, 8700, 8703, 8704, 8705, 8706,
    8708, 8711, 8712, 8713, 8714, 8715, 8716, 8718, 8721, 8722, 8723, 8724, 8725, 8726, 8728, 8729,
    8730, 8733, 8736, 8737, 8739, 8740, 8741, 8745, 8750, 8752, 8753, 8755, 8756, 8758, 8760, 8762,
    8763, 8764, 8765, 8768, 8770, 8771, 8774, 8778, 8779, 8783, 8785, 8790, 8791, 8792, 8793, 8794,
    8798, 8799, 8801, 8804, 8806, 8807, 8812, 8814, 8815, 8817, 8818, 8820, 8821, 8822, 8823, 8824,
    8825, 8826, 8827, 8828, 8831, 8832, 8834, 8838, 8839, 8844, 8845, 8846, 8847, 8848, 8849, 8850,
    8853, 8854, 8855, 8858, 8860, 8863, 8870, 8872, 8874, 8875, 8878, 8879, 8882, 8885, 8886, 8888,
    8889, 8891, 8892, 8893, 8894, 8895, 8896, 8897, 8898, 8901, 8904, 8905, 8907, 8910, 8911, 8915,
    8916, 8917, 8918, 8919, 8920, 8921, 8923, 8924, 8925, 8926, 8927, 8928, 8930, 8931, 8932, 8934,
    8935, 8939, 8942, 8944, 8947, 8948, 8949, 8950, 8952, 8955, 8957, 8964, 8966, 8968, 8970, 8971,
    8972, 8974, 8981, 8983, 8985, 8991, 8992, 8994, 8995, 8996, 9002, 9004, 9005, 9006, 9007, 9008,
    9009, 9010, 9011, 9014, 9015, 9017, 9018, 9019, 9020, 9023, 9025, 9027, 9029, 9031, 9032, 9033,
    9038, 9040, 9042, 9045, 9046, 9047, 9048, 9049, 9050, 9051, 9052, 9055, 9059, 9060, 9061, 9062,
    9068, 9069, 9070, 9076, 9078, 9080, 9084, 9085, 9087, 9089, 9090, 9093, 9097, 9100, 9105, 9107,
    9110, 9111, 9112, 9114, 9115, 9116, 9119, 9120, 9125, 9126, 9127, 9129, 9131, 9132, 9133, 9134,
    9135, 9137, 9138, 9139, 9140, 9141, 9143, 9145, 9150, 9151, 9157, 9158, 9159, 9161, 9163, 9164,
    9167, 9168, 9171, 9173, 9174, 9176, 9177, 9179, 9180, 9181, 9183, 9184, 9185, 9186, 9189, 9195,
    9196, 9197, 9198, 9200, 9202, 9203, 9204, 9206, 9208, 9210, 9212, 9213, 9214, 9221, 9222, 9224,
    9228, 9229, 9230, 9234, 9236, 9237, 9245, 9246, 9248, 9250, 9255, 9256, 9257, 9258, 9259, 9263,
    9265, 9266, 9267, 9268, 9269, 9276, 9277, 9283, 9286, 9287, 9289, 9291, 9292, 9298, 9299, 9300,
    9303, 9304, 9305, 9306, 9307, 9308, 9311, 9316, 9317, 9318, 9319, 9320, 9322, 9323, 9324, 9325,
    9327, 9328, 9334, 9335, 9338, 9340, 9342, 9344, 9346, 9349, 9351, 9354, 9356, 9357, 9362, 9364,
    9365, 9366, 9367, 9368, 9369, 9370, 9374, 9375, 9376, 9377, 9381, 9385, 9387, 9388, 9390, 9391,
    9392, 9398, 9399, 9400, 9401, 9402, 9404, 9407, 9409, 9410, 9412, 9413, 9414, 9419, 9421, 9423,
    9424, 9426, 9427, 9429, 9434, 9435, 9437, 9440, 9443, 9444, 9445, 9447, 9448, 9452, 9456, 9457,
    9459, 9460, 9462, 9464, 9469, 9470, 9471, 9472, 9473, 9474, 9476, 9477, 9478, 9479, 9480, 9483,
    9484, 9486, 9489, 9490, 9494, 9495, 9496, 9498, 9500, 9501, 9502, 9504, 9505, 9506, 9507, 9508,
    9509, 9510, 9511, 9512, 9513, 9515, 9516, 9517, 9518, 9522, 9523, 9526, 9527, 9534, 9536, 9537,
    9541, 9542, 9544, 9545, 9547, 9551, 9552, 9553, 9556, 9558, 9559, 9563, 9566, 9568, 9571, 9574,
    9577, 9579, 9582, 9584, 9589, 9593, 9599, 9600, 9603, 9606, 9608, 9609, 9610, 9614, 9615, 9616,
    9619, 9621, 9622, 9623, 9626, 9627, 9628, 9629, 9632, 9633, 9634, 9635, 9640, 9641, 9643, 9644,
    9646, 9651, 9653, 9654, 9656, 9657, 9661, 9662, 9666, 9668, 9672, 9673, 9675, 9679, 9680, 9681,
    9684, 9685, 9687, 9688, 9690, 9693, 9694, 9696, 9698, 9699, 9700, 9702, 9706, 9707, 9712, 9715,
    9717, 9718, 9721, 9722, 9724, 9725, 9729, 9732, 9733, 9735, 9737, 9738, 9739, 9740, 9741, 9747,
    9748, 9749, 9750, 9751, 9752, 9753, 9759, 9765, 9769, 9770, 9771, 9776, 9778, 9779, 9780, 9781,
    9783, 9784, 9786, 9787, 9789, 9790, 9791, 9793, 9796, 9801, 9802, 9803, 9805, 9806, 9812, 9816,
    9817, 9819, 9827, 9828, 9829, 9832, 9836, 9838, 9839, 9840, 9851, 9856, 9857, 9858, 9861, 9862,
    9864, 9865, 9866, 9867, 9871, 9872, 9873, 9877, 9878, 9880, 9881, 9882, 9886, 9894, 9895, 9897,
    9898, 9899, 9901, 9902, 9903, 9906, 9907, 9910, 9912, 9914, 9915, 9917, 9918, 9919, 9920, 9921,
    9922, 9923, 9924, 9925, 9930, 9932, 9938, 9940, 9941, 9942, 9943, 9944, 9945, 9947, 9948, 9950,
    9951, 9953, 9954, 9957, 9958, 9959, 9960, 9961, 9962, 9964, 9966, 9967, 9970, 9971, 9972, 9974,
    9977, 9978, 9979, 9981, 9983, 9986, 9989, 9990, 9991, 9993, 9994, 9995, 9997, 9998, 9999,
];
