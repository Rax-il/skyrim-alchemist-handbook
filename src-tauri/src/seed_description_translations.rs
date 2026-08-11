// seed_description_translations.rs — литературный перевод описаний
// ингредиентов на не-русские языки. В отличие от seed_translations.rs
// (официальные строки Bethesda, сгенерировано пайплайном tools/i18n,
// НЕ редактировать вручную) — этот файл содержит перевод, сделанный
// вручную (Claude), никакого отношения к официальным строкам игры не
// имеющий. Редактируется вручную, партиями — см.
// docs/superpowers/specs/2026-08-10-i18n-storage-design.md, дополнение
// "перевод описаний ингредиентов на 8 языков".

/// Переводы описаний 110 обычных ингредиентов (seed_data.rs::MEDIA) —
/// (ru_name, lang, описание). Наполняется постепенно, отдельными партиями
/// — см. design doc. Первая партия (2026-08-11): 15 ингредиентов, от
/// "Виноград джазби" до "Зубы ледяного привидения" (первые 15 по порядку в
/// MEDIA). Для ru_name без записи migrate_i18n() корректно возвращает
/// description = '' (не ошибка, не мусор) — так и должно быть для
/// оставшихся, ещё не переведённых партий.
pub const DESCRIPTION_TRANSLATIONS: &[(&str, &str, &str)] = &[
    ("Виноград джазби", "de", "Trauben, die direkt auf Felsen und Gestein wachsen — am häufigsten rund um das Dampfsengerlager, den Schrein der Alten und den Atronach-Stein in Ostmark. Der Legende nach war es einst ein Verbrechen gegen den Staat, eine Traube ohne die Erlaubnis des Kaisers zu pflücken."),
    ("Виноград джазби", "en", "Grapes that grow directly on rocks and stone — most abundant around Steamscorch Camp, the Shrine of the Ancients, and the Atronach Stone in Eastmarch. Legend has it that picking a bunch without the Emperor's permission was once a crime against the state."),
    ("Виноград джазби", "es", "Uvas que crecen directamente sobre rocas y piedras — abundan especialmente alrededor del Campamento Achicharrado, el Santuario de los Antiguos y la Piedra del Atronach en Marca Oriental. Cuenta la leyenda que arrancar un racimo sin el permiso del Emperador fue en su día un crimen contra el estado."),
    ("Виноград джазби", "fr", "Des raisins qui poussent directement sur la roche et la pierre — on les trouve surtout autour du Camp de la Vapeur Brûlante, du Sanctuaire des Anciens et de la Pierre de l'Atronach en Marche de l'Est. La légende raconte que cueillir une grappe sans la permission de l'Empereur était autrefois un crime contre l'État."),
    ("Виноград джазби", "it", "Uva che cresce direttamente su rocce e pietre — più diffusa nei pressi dell'Accampamento Bruciavapore, del Santuario degli Antichi e della Pietra dell'Atronach a Marcaorientale. La leggenda narra che cogliere un grappolo senza il permesso dell'Imperatore fosse un tempo un crimine contro lo stato."),
    ("Виноград джазби", "ja", "岩や石の上に直接生える葡萄——イーストマーチのスチームスコーチ野営地、古の者の祠、アトロナクの石の周辺に多く見られる。伝説によれば、かつて皇帝の許しなく房を摘むことは国家に対する犯罪だったという。"),
    ("Виноград джазби", "pl", "Winogrona rosnące wprost na skałach i kamieniach — najczęściej spotykane wokół Obozu Parowego Poparzenia, Świątyni Starożytnych i Kamienia Atronacha we Wschodniej Marchii. Legenda głosi, że zerwanie kiści bez zgody Cesarza było niegdyś zbrodnią przeciw państwu."),
    ("Виноград джазби", "zh-Hant", "直接生長在岩石與石塊上的葡萄——大多分布於東陲的蒸氣灼燒營地、古人聖壇與亞龍納克石周圍。傳說中,未經皇帝許可摘採一串葡萄,曾是一項叛國重罪。"),

    ("Вредозобник", "de", "Eine niedrig wachsende Pflanze mit dunkelroten, stacheligen Blättern, hinzugefügt in der Erweiterung Dragonborn. Wächst im Süden von Solstheim, besonders zahlreich rund um Rabenfels; wird von den Rieklingen in ihren Ritualen verwendet."),
    ("Вредозобник", "en", "A low-growing plant with dark-red, spiny leaves, added in the Dragonborn add-on. Grows in the southern part of Solstheim, especially abundant around Raven Rock; used by the Rieklings in their rituals."),
    ("Вредозобник", "es", "Una planta de poca altura con hojas espinosas de color rojo oscuro, añadida en la ampliación Dragonborn. Crece en el sur de Solstheim, especialmente abundante alrededor de Roca Cuervo; utilizada por los riekling en sus rituales."),
    ("Вредозобник", "fr", "Une plante basse aux feuilles épineuses rouge sombre, ajoutée dans l'extension Dragonborn. Pousse dans le sud de Solstheim, particulièrement abondante autour de Rocherfreux ; utilisée par les Rieklings dans leurs rituels."),
    ("Вредозобник", "it", "Una pianta bassa dalle foglie spinose rosso scuro, aggiunta nel componente aggiuntivo Dragonborn. Cresce nella parte meridionale di Solstheim, particolarmente diffusa intorno a Roccacorvo; utilizzata dai riekling nei loro rituali."),
    ("Вредозобник", "ja", "暗赤色の棘のある葉を持つ、丈の低い植物。拡張コンテンツ「ドラゴンボーン」で追加された。ソルスセイム島の南部、特にレイヴンロックの周辺に多く自生し、リークリングたちが儀式に用いる。"),
    ("Вредозобник", "pl", "Niska roślina o ciemnoczerwonych, kolczastych liściach, dodana w dodatku Dragonborn. Rośnie na południu Solstheim, szczególnie licznie wokół Kruczej Skały; wykorzystywana przez rieklingów w ich rytuałach."),
    ("Вредозобник", "zh-Hant", "一種葉片深紅帶刺的矮小植物,隨「龍裔」擴充內容加入。生長於索瑟海姆島南部,尤以鴉石城周圍最為茂盛;瑞克林人會在儀式中使用它。"),

    ("Гигантский лишайник", "de", "Eine Flechte, die auf Felsen und Mauern an feuchten Orten Himmelsrands wächst — eine verbreitete und leicht zu findende Zutat."),
    ("Гигантский лишайник", "en", "A lichen that grows on rocks and walls in the damp places of Skyrim — a common and easily gathered ingredient."),
    ("Гигантский лишайник", "es", "Un liquen que crece sobre rocas y muros en los lugares húmedos de Skyrim — un ingrediente común y fácil de recolectar."),
    ("Гигантский лишайник", "fr", "Un lichen qui pousse sur les rochers et les murs, dans les endroits humides de Bordeciel — un ingrédient courant et facile à récolter."),
    ("Гигантский лишайник", "it", "Un lichene che cresce su rocce e muri nei luoghi umidi di Skyrim — un ingrediente comune e facile da raccogliere."),
    ("Гигантский лишайник", "ja", "スカイリムの湿った場所の岩や壁に生える地衣類——ありふれていて採取しやすい素材である。"),
    ("Гигантский лишайник", "pl", "Porost rosnący na skałach i murach w wilgotnych miejscach Skyrim — pospolity i łatwy do zebrania składnik."),
    ("Гигантский лишайник", "zh-Hant", "生長在天際潮濕之處岩石與牆面上的地衣——常見且容易採集的材料。"),

    ("Глаз саблезуба", "de", "Wird von erlegten Säbelzahnkatzen gewonnen, die in ganz Himmelsrand vorkommen, besonders in dessen südlichen und westlichen Regionen."),
    ("Глаз саблезуба", "en", "Harvested from slain saber cats, found throughout Skyrim, especially in its southern and western regions."),
    ("Глаз саблезуба", "es", "Se obtiene de gatos dientes de sable abatidos, que se encuentran por todo Skyrim, especialmente en sus regiones meridionales y occidentales."),
    ("Глаз саблезуба", "fr", "Récupéré sur des chats-sabres abattus, que l'on trouve dans tout Bordeciel, en particulier dans ses régions méridionales et occidentales."),
    ("Глаз саблезуба", "it", "Si ottiene da felini dai denti a sciabola uccisi, presenti in tutto Skyrim, specialmente nelle sue regioni meridionali e occidentali."),
    ("Глаз саблезуба", "ja", "スカイリム各地、特に南部と西部の地域に生息するセイバーキャットを倒して手に入る。"),
    ("Глаз саблезуба", "pl", "Pozyskiwane z zabitych szablozębnych kotów, spotykanych w całym Skyrim, zwłaszcza w jego południowych i zachodnich regionach."),
    ("Глаз саблезуба", "zh-Hant", "取自被擊殺的劍齒貓,牠們遍布天際各地,尤以南部與西部地區居多。"),

    ("Гниль Намиры", "de", "Eine Zutat, die mit dem Daedrafürsten Namira verbunden ist, der Schutzherrin alles Verfaulten und Widerwärtigen — zu finden in Höhlen und unreinen Orten, auch in der Nähe ihres Schreins in der Reachwater-Höhle."),
    ("Гниль Намиры", "en", "An ingredient tied to the Daedric Prince Namira, patron of all that is rotten and vile — found in caves and unclean places, including near her shrine in Reachwater Rock."),
    ("Гниль Намиры", "es", "Un ingrediente ligado a la Príncipe Daédrica Namira, patrona de todo lo podrido y repugnante — se encuentra en cuevas y lugares impuros, incluso cerca de su santuario en la Gruta de Reachwater."),
    ("Гниль Намиры", "fr", "Un ingrédient lié au Prince Daedrique Namira, patronne de tout ce qui est pourri et répugnant — que l'on trouve dans les grottes et les lieux impurs, y compris près de son sanctuaire dans la grotte de Reachwater."),
    ("Гниль Намиры", "it", "Un ingrediente legato al Principe Daedrico Namira, patrona di tutto ciò che è marcio e ripugnante — si trova nelle caverne e nei luoghi impuri, anche nei pressi del suo santuario nella grotta di Reachwater."),
    ("Гниль Намиры", "ja", "腐敗と忌まわしきものすべての守護者たるデイドラ公ナミラに関わる素材——洞窟や不浄の地に見られ、リーチウォーター岩屋にある彼女の祠の付近にも存在する。"),
    ("Гниль Намиры", "pl", "Składnik związany z Daedrycznym Księciem Namirą, patronką wszystkiego, co zgniłe i odrażające — spotykany w jaskiniach i nieczystych miejscach, także w pobliżu jej świątyni w Grocie Reachwater."),
    ("Гниль Намиры", "zh-Hant", "與魔神納米拉有關的材料,她是一切腐朽與可憎之物的守護者——見於洞窟與不潔之地,包括她位於礁水岩洞聖壇的附近。"),

    ("Двемерское масло", "de", "Ein Öl, das man in dwemerischen Ruinen findet — es wurde von der alten Rasse der Dwemer zum Schmieren ihrer Maschinen und Automatonen verwendet."),
    ("Двемерское масло", "en", "An oil found in Dwemer ruins — used by the ancient Dwemer race to lubricate their machinery and automatons."),
    ("Двемерское масло", "es", "Un aceite que se encuentra en las ruinas dwemer — utilizado por la antigua raza dwemer para lubricar su maquinaria y sus autómatas."),
    ("Двемерское масло", "fr", "Une huile que l'on trouve dans les ruines dwemer — utilisée par l'ancienne race des Dwemers pour lubrifier leurs machines et leurs automates."),
    ("Двемерское масло", "it", "Un olio che si trova nelle rovine dwemer — utilizzato dall'antica razza dei Dwemer per lubrificare i loro macchinari e automi."),
    ("Двемерское масло", "ja", "ドゥーマー遺跡で見つかる油——古代種族ドゥーマーが機械やオートマトンの潤滑に用いていたものである。"),
    ("Двемерское масло", "pl", "Olej znajdowany w dwemerskich ruinach — używany przez starożytną rasę Dwemerów do smarowania ich maszyn i automatonów."),
    ("Двемерское масло", "zh-Hant", "在錠莫爾遺跡中發現的油——古老的錠莫爾種族曾用它潤滑機械與自動機關。"),

    ("Древесина горелого сприггана", "de", "Eine Zutat aus der Erweiterung Dragonborn: verkohltes Holz, das von einem durch Feuer getöteten Spriggan zurückbleibt. Zu finden auf der Insel Solstheim."),
    ("Древесина горелого сприггана", "en", "An ingredient from the Dragonborn add-on: charred wood left behind by a spriggan slain by fire. Found on the island of Solstheim."),
    ("Древесина горелого сприггана", "es", "Un ingrediente de la ampliación Dragonborn: madera carbonizada que queda de un espíritu del bosque abatido por el fuego. Se encuentra en la isla de Solstheim."),
    ("Древесина горелого сприггана", "fr", "Un ingrédient de l'extension Dragonborn : bois calciné laissé par un spriggan tué par le feu. Que l'on trouve sur l'île de Solstheim."),
    ("Древесина горелого сприггана", "it", "Un ingrediente del componente aggiuntivo Dragonborn: legno carbonizzato che rimane da uno spriggan ucciso dal fuoco. Si trova sull'isola di Solstheim."),
    ("Древесина горелого сприггана", "ja", "拡張コンテンツ「ドラゴンボーン」の素材——火によって倒されたスプリガンの残した焦げた木材。ソルスセイム島で見つかる。"),
    ("Древесина горелого сприггана", "pl", "Składnik z dodatku Dragonborn: zwęglone drewno pozostałe po spriggan zabitym ogniem. Spotykany na wyspie Solstheim."),
    ("Древесина горелого сприггана", "zh-Hant", "來自「龍裔」擴充內容的材料:被火焰擊殺的樹精所留下的焦木。見於索瑟海姆島。"),

    ("Жареная злокрысья кожа", "de", "Rattenlingshaut, über dem Feuer geröstet — zu finden in der Nähe von Lagerfeuern und Lagern."),
    ("Жареная злокрысья кожа", "en", "Skeever hide, roasted over a fire — can be found near campfires and camps."),
    ("Жареная злокрысья кожа", "es", "Piel de rata gigante asada al fuego — se puede encontrar cerca de hogueras y campamentos."),
    ("Жареная злокрысья кожа", "fr", "Peau de charognard rôtie sur le feu — se trouve près des feux de camp et des campements."),
    ("Жареная злокрысья кожа", "it", "Pelle di ratto gigante arrostita sul fuoco — si trova vicino ai fuochi da campo e agli accampamenti."),
    ("Жареная злокрысья кожа", "ja", "火であぶられたスキーバーの皮——焚き火や野営地の近くで見つかる。"),
    ("Жареная злокрысья кожа", "pl", "Skóra złoszczura, upieczona na ogniu — można ją znaleźć w pobliżu ognisk i obozowisk."),
    ("Жареная злокрысья кожа", "zh-Hant", "以火烤過的害鼠皮——可在營火與營地附近找到。"),

    ("Желе нетча", "de", "Eine Zutat aus der Erweiterung Dragonborn, gewonnen von Netchs — großen, schwebenden, gallertartigen Kreaturen, die auf der Insel Solstheim leben."),
    ("Желе нетча", "en", "An ingredient from the Dragonborn add-on, harvested from netches — large, floating, jelly-like creatures found on the island of Solstheim."),
    ("Желе нетча", "es", "Un ingrediente de la ampliación Dragonborn, obtenido de los netch — grandes criaturas gelatinosas y flotantes que habitan en la isla de Solstheim."),
    ("Желе нетча", "fr", "Un ingrédient de l'extension Dragonborn, récolté sur les netchs — de grandes créatures gélatineuses flottantes vivant sur l'île de Solstheim."),
    ("Желе нетча", "it", "Un ingrediente del componente aggiuntivo Dragonborn, ottenuto dai netch — grandi creature gelatinose fluttuanti che vivono sull'isola di Solstheim."),
    ("Желе нетча", "ja", "拡張コンテンツ「ドラゴンボーン」の素材——ソルスセイム島に生息する、浮遊するゼリー状の巨大生物ネッチから得られる。"),
    ("Желе нетча", "pl", "Składnik z dodatku Dragonborn, pozyskiwany z netchów — dużych, unoszących się, galaretowatych stworzeń zamieszkujących wyspę Solstheim."),
    ("Желе нетча", "zh-Hant", "來自「龍裔」擴充內容的材料,取自涅奇——一種棲息於索瑟海姆島、體型巨大且會漂浮的果凍狀生物。"),

    ("Желе пепельного прыгуна", "de", "Eine Zutat aus der Erweiterung Dragonborn, gewonnen von Aschehüpfern — großen Insekten, die in den Aschelanden von Solstheim leben."),
    ("Желе пепельного прыгуна", "en", "An ingredient from the Dragonborn add-on, obtained from ash hoppers — large insects that inhabit the ashen lands of Solstheim."),
    ("Желе пепельного прыгуна", "es", "Un ingrediente de la ampliación Dragonborn, obtenido de los saltacenizas — grandes insectos que habitan las tierras de ceniza de Solstheim."),
    ("Желе пепельного прыгуна", "fr", "Un ingrédient de l'extension Dragonborn, obtenu à partir des sauteurs des cendres — de grands insectes vivant dans les terres cendrées de Solstheim."),
    ("Желе пепельного прыгуна", "it", "Un ingrediente del componente aggiuntivo Dragonborn, ottenuto dai saltatori di cenere — grandi insetti che abitano le terre di cenere di Solstheim."),
    ("Желе пепельного прыгуна", "ja", "拡張コンテンツ「ドラゴンボーン」の素材——ソルスセイムの灰の大地に生息する大型昆虫、アッシュホッパーから得られる。"),
    ("Желе пепельного прыгуна", "pl", "Składnik z dodatku Dragonborn, pozyskiwany ze skoczków popiołowych — dużych owadów zamieszkujących popielne ziemie Solstheim."),
    ("Желе пепельного прыгуна", "zh-Hant", "來自「龍裔」擴充內容的材料,取自灰蝗——一種棲息於索瑟海姆灰燼大地的大型昆蟲。"),

    ("Жёлтый горноцвет", "de", "Eine Bergblume, hinzugefügt in der Erweiterung Dawnguard — zu finden neben blauen, violetten und roten Bergblumen in ganz Himmelsrand."),
    ("Жёлтый горноцвет", "en", "A mountain flower added in the Dawnguard add-on — found alongside blue, purple, and red mountain flowers throughout Skyrim."),
    ("Жёлтый горноцвет", "es", "Una flor de montaña añadida en la ampliación Dawnguard — se encuentra junto a las flores de montaña azules, moradas y rojas por todo Skyrim."),
    ("Жёлтый горноцвет", "fr", "Une fleur de montagne ajoutée dans l'extension Dawnguard — que l'on trouve aux côtés des fleurs de montagne bleues, violettes et rouges dans tout Bordeciel."),
    ("Жёлтый горноцвет", "it", "Un fiore di montagna aggiunto nel componente aggiuntivo Dawnguard — si trova insieme ai fiori di montagna blu, viola e rossi in tutto Skyrim."),
    ("Жёлтый горноцвет", "ja", "拡張コンテンツ「ドウンガード」で追加された山の花——スカイリム各地で青、紫、赤の山の花と並んで見られる。"),
    ("Жёлтый горноцвет", "pl", "Kwiat górski dodany w dodatku Dawnguard — spotykany obok niebieskiego, fioletowego i czerwonego kwiatu górskiego w całym Skyrim."),
    ("Жёлтый горноцвет", "zh-Hant", "隨「黎明守衛」擴充內容加入的山花——與藍色、紫色及紅色山花一同見於天際各地。"),

    ("Жемчужина", "de", "Wird aus Weichtierschalen gewonnen, die in den Flüssen und Seen Himmelsrands vorkommen, manchmal auch bei Schlammkrabben."),
    ("Жемчужина", "en", "Harvested from mollusk shells found in the rivers and lakes of Skyrim, and sometimes from mudcrabs as well."),
    ("Жемчужина", "es", "Se obtiene de conchas de moluscos que aparecen en los ríos y lagos de Skyrim, y a veces también en los cangrejos de fango."),
    ("Жемчужина", "fr", "Récupérée dans les coquilles de mollusques que l'on trouve dans les rivières et les lacs de Bordeciel, et parfois aussi chez les crabes de boue."),
    ("Жемчужина", "it", "Si ottiene dai gusci di molluschi che si trovano nei fiumi e nei laghi di Skyrim, e talvolta anche nei granchi di fango."),
    ("Жемчужина", "ja", "スカイリムの川や湖に生息する軟体動物の殻から採れる。時にはマッドクラブからも得られる。"),
    ("Жемчужина", "pl", "Pozyskiwana z muszli mięczaków spotykanych w rzekach i jeziorach Skyrim, a czasem także u krabów błotnych."),
    ("Жемчужина", "zh-Hant", "取自天際河流與湖泊中軟體動物的貝殼,有時也能在泥蟹身上找到。"),

    ("Живица сприггана", "de", "Eine harzige Flüssigkeit, die man erhält, indem man einen Spriggan tötet — ein baumähnliches Waldwesen, das die Natur Himmelsrands beschützt."),
    ("Живица сприггана", "en", "A resinous fluid obtained by slaying a spriggan — a tree-like forest creature that protects the nature of Skyrim."),
    ("Живица сприггана", "es", "Un fluido resinoso que se obtiene matando a un espíritu del bosque — una criatura arbórea que protege la naturaleza de Skyrim."),
    ("Живица сприггана", "fr", "Un liquide résineux que l'on obtient en tuant un spriggan — une créature sylvestre semblable à un arbre, protectrice de la nature de Bordeciel."),
    ("Живица сприггана", "it", "Un fluido resinoso che si ottiene uccidendo uno spriggan — una creatura arborea della foresta che protegge la natura di Skyrim."),
    ("Живица сприггана", "ja", "スプリガンを倒すことで得られる樹脂状の液体——スカイリムの自然を守る、木のような森の生物である。"),
    ("Живица сприггана", "pl", "Żywiczna ciecz uzyskiwana z zabicia spriggan — drzewopodobnej leśnej istoty chroniącej naturę Skyrim."),
    ("Живица сприггана", "zh-Hant", "殺死樹精後可獲得的樹脂狀液體——樹精是守護天際自然的樹形森林生物。"),

    ("Жир тролля", "de", "Wird von erlegten Trollen gewonnen, die in den gebirgigen und nördlichen Regionen Himmelsrands anzutreffen sind."),
    ("Жир тролля", "en", "Harvested from slain trolls, which can be found in the mountainous and northern regions of Skyrim."),
    ("Жир тролля", "es", "Se obtiene de trolls abatidos, que pueden encontrarse en las regiones montañosas y septentrionales de Skyrim."),
    ("Жир тролля", "fr", "Récupérée sur des trolls abattus, que l'on trouve dans les régions montagneuses et septentrionales de Bordeciel."),
    ("Жир тролля", "it", "Si ottiene da troll uccisi, che si possono incontrare nelle regioni montuose e settentrionali di Skyrim."),
    ("Жир тролля", "ja", "スカイリムの山岳地帯や北部地域に生息するトロールを倒すことで手に入る。"),
    ("Жир тролля", "pl", "Pozyskiwany z zabitych trolli, które można spotkać w górskich i północnych regionach Skyrim."),
    ("Жир тролля", "zh-Hant", "取自被擊殺的巨魔,牠們可見於天際的山區與北部地區。"),

    ("Зубы ледяного привидения", "de", "Wird von erlegten Eisgeistern gewonnen — Geistern aus Eis, die in den kältesten, verschneitesten Winkeln Himmelsrands leben."),
    ("Зубы ледяного привидения", "en", "Harvested from slain ice wraiths — spirits of ice that dwell in the coldest, most snow-covered corners of Skyrim."),
    ("Зубы ледяного привидения", "es", "Se obtienen de espectros de hielo abatidos — espíritus de hielo que habitan en los rincones más fríos y nevados de Skyrim."),
    ("Зубы ледяного привидения", "fr", "Récupérées sur des spectres des glaces abattus — des esprits de glace qui vivent dans les recoins les plus froids et enneigés de Bordeciel."),
    ("Зубы ледяного привидения", "it", "Si ottengono da spettri di ghiaccio uccisi — spiriti di ghiaccio che vivono negli angoli più freddi e innevati di Skyrim."),
    ("Зубы ледяного привидения", "ja", "討伐したアイス・レイスから得られる——スカイリムの最も寒く雪深い一角に棲む氷の精霊である。"),
    ("Зубы ледяного привидения", "pl", "Pozyskiwane z zabitych zjaw lodu — duchów lodu zamieszkujących najzimniejsze, najbardziej ośnieżone zakątki Skyrim."),
    ("Зубы ледяного привидения", "zh-Hant", "取自被擊殺的冰魄——棲息於天際最寒冷、白雪覆蓋角落的冰霜幽靈。"),
];

/// Переводы имени для случаев, когда официального совпадения от Plan A не
/// нашлось — сейчас единственный случай: "Смертная плоть" (Plague of the
/// Dead), плагин отсутствовал в скачанном архиве официальных строк. Тот же
/// формат и та же роль в migrate_i18n(), что и TRANSLATIONS
/// (seed_translations.rs), просто источник — не официальный пайплайн.
pub const NAME_OVERRIDES: &[(&str, &str, &str)] = &[
    ("Смертная плоть", "de", "Fleisch der Toten"),
    ("Смертная плоть", "en", "Flesh of the Dead"),
    ("Смертная плоть", "es", "Carne de los muertos"),
    ("Смертная плоть", "fr", "Chair des morts"),
    ("Смертная плоть", "it", "Carne dei morti"),
    ("Смертная плоть", "ja", "死者の肉"),
    ("Смертная плоть", "pl", "Ciało zmarłego"),
    ("Смертная плоть", "zh-Hant", "死者之肉"),
];

/// Общее описание для всех 52 ингредиентов дополнения Rare Curios —
/// (lang, описание). Ru-оригинал: rare_curios.rs::RARE_CURIOS_DESCRIPTION.
pub const RARE_CURIOS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein seltener Fund von jenseits Himmelsrands: Diese Zutat gelangt dank der Erweiterung Rare Curios in die Hände des Drachenbluts — Khajiit-Händler bringen solche Kuriositäten in ihren Karawanen aus fernen Landen: aus Cyrodiil, Morrowind und den Wogenden Inseln."),
    ("en", "A rare find from beyond Skyrim: this ingredient reaches the Dragonborn's hands thanks to the Rare Curios add-on — Khajiit merchants bring such curiosities in their caravans from distant lands: Cyrodiil, Morrowind, and the Shivering Isles."),
    ("es", "Un hallazgo poco común llegado de más allá de Skyrim: este ingrediente llega a manos del Sangre de Dragón gracias a la ampliación Rare Curios — los mercaderes Khajiitas traen estas rarezas en sus caravanas desde tierras lejanas: Cyrodiil, Morrowind y las Islas Trémulas."),
    ("fr", "Une trouvaille rare venue d'au-delà de Bordeciel : cet ingrédient parvient entre les mains du Dovahkiin grâce à l'extension Rare Curios — des marchands khajiits apportent ces curiosités dans leurs caravanes depuis des terres lointaines : Cyrodiil, Morrowind et les Îles de la Démence."),
    ("it", "Un raro reperto da oltre i confini di Skyrim: questo ingrediente giunge nelle mani del Sangue di Drago grazie al componente aggiuntivo Rare Curios — i mercanti Khajiit portano queste curiosità nelle loro carovane da terre lontane: Cyrodiil, Morrowind e le Isole Tremule."),
    ("ja", "スカイリムの外からもたらされた稀少な品——この素材は「レア・キュリオス」拡張のおかげでドラゴンボーンの手に渡る。カジート商人たちがオブリビオン、モロウウィンド、シバリング・アイルズなど遠い地から、隊商にこうした珍品を運んでくる。"),
    ("pl", "Rzadkie znalezisko spoza granic Skyrim: ten składnik trafia w ręce Smoczej Krwi dzięki dodatkowi Rare Curios — kupcy Khajiit przywożą takie osobliwości w swoich karawanach z dalekich krain: Cyrodiil, Morrowind i Drżących Wysp."),
    ("zh-Hant", "來自天際境外的稀有發現:多虧「稀奇珍玩」擴充內容,這件材料才得以落入龍裔之手——凱季特商人將這些奇珍異寶從奧不里維恩、莫羅溫德與顫慄島等遙遠之地,經由商隊運來。"),
];

/// Общее описание для всех ингредиентов творения "Рыбалка" — (lang, описание).
/// Ru-оригинал: creations.rs::FISHING_DESC.
pub const FISHING_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein Fang aus der Erweiterung „Angeln\": Dieser Bewohner von Himmelsrands Flüssen, Seen und Meeren beißt bei jedem Angler an, der bereit ist, stundenlang am Wasser auf einen Biss zu warten — und wird dafür nicht nur mit Beute, sondern auch mit alchemistischem Wert belohnt."),
    ("en", "A catch from the Fishing add-on: this denizen of Skyrim's rivers, lakes, and seas takes the hook of any angler willing to sit by the water for hours waiting for a bite — rewarding them with more than just a catch, but alchemical value as well."),
    ("es", "Una captura de la ampliación «Pesca»: este habitante de los ríos, lagos y mares de Skyrim muerde el anzuelo de todo pescador dispuesto a esperar horas junto al agua — recompensándolo no solo con una presa, sino también con valor alquímico."),
    ("fr", "Une prise de l'extension « Pêche » : cet habitant des rivières, des lacs et des mers de Bordeciel mord à l'hameçon du pêcheur prêt à patienter des heures au bord de l'eau — récompensant sa patience non seulement par une prise, mais aussi par une valeur alchimique."),
    ("it", "Una cattura del componente aggiuntivo «Pesca»: questo abitante dei fiumi, dei laghi e dei mari di Skyrim abbocca all'amo di ogni pescatore disposto a sedersi per ore in attesa di un abboccamento — ricompensandolo non solo con una preda, ma anche con un valore alchemico."),
    ("ja", "「釣り」拡張の釣果——スカイリムの川、湖、海に棲むこの生き物は、何時間も水辺に座って当たりを待つ辛抱強い釣り人の針にかかる。得られるのは獲物だけでなく、錬金術的な価値でもある。"),
    ("pl", "Połów z dodatku „Wędkarstwo\": ten mieszkaniec rzek, jezior i mórz Skyrim łapie się na haczyk każdego wędkarza gotowego siedzieć godzinami nad wodą w oczekiwaniu na branie — nagradzając go nie tylko zdobyczą, ale i wartością alchemiczną."),
    ("zh-Hant", "「釣魚」擴充內容的漁獲:這種棲息在天際河流、湖泊與海洋中的生物,會咬上願意在水邊耐心等候數小時的釣客的魚鉤——回報他的不僅是漁獲,還有煉金術上的價值。"),
];

/// Общее описание для всех ингредиентов творения "Святые и соблазнители" —
/// (lang, описание). Ru-оригинал: creations.rs::SAINTS_DESC.
pub const SAINTS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Eine Kuriosität jenseits der Realität: Diese Zutat steht in Verbindung mit der Kreation „Heilige und Verführer\" — einer Geschichte über eine Reise in Sheogoraths seltsames, verdrehtes Reich, wo selbst Pflanzen und Insekten der Logik des Wahnsinnsprinzen gehorchen."),
    ("en", "A curiosity from beyond reality: this ingredient is tied to the Saints & Seducers creation — a tale of a journey into the strange, twisted realm of Sheogorath, where even plants and insects obey the logic of the Mad Prince."),
    ("es", "Una rareza más allá de la realidad: este ingrediente está ligado a la creación «Santos y Seductores» — una historia sobre un viaje al extraño y retorcido reino de Sheogorath, donde hasta las plantas y los insectos obedecen la lógica del Príncipe Loco."),
    ("fr", "Une curiosité venue d'ailleurs : cet ingrédient est lié à la création « Saints et Séducteurs » — une histoire de voyage dans le royaume étrange et tordu de Sheogorath, où même les plantes et les insectes obéissent à la logique du Prince Fou."),
    ("it", "Una curiosità che sfida la realtà: questo ingrediente è legato alla creazione «Santi e Seduttori» — una storia di viaggio nel regno strano e contorto di Sheogorath, dove persino le piante e gli insetti obbediscono alla logica del Principe Folle."),
    ("ja", "現実の外からやってきた珍品——この素材は「聖者と誘惑者」というクリエーションに関わりがある。狂気の公シェオゴラスの支配する、奇妙でねじれた領域への旅の物語であり、そこでは草木や虫さえも狂気の公の理屈に従う。"),
    ("pl", "Osobliwość spoza rzeczywistości: ten składnik związany jest z kreacją „Święci i Uwodziciele\" — opowieścią o podróży do dziwnego, wykrzywionego królestwa Sheogoratha, gdzie nawet rośliny i owady podlegają logice Szalonego Księcia."),
    ("zh-Hant", "超脫現實的奇珍:這件材料與創作內容「聖徒與誘惑者」有關——講述一段前往希歐格拉斯那奇異扭曲領地的旅程,在那裡即使是花草蟲豸,也遵循著這位瘋狂親王的邏輯。"),
];

/// Общее описание для ингредиента творения "Чума мертвецов" —
/// (lang, описание). Ru-оригинал: creations.rs::PLAGUE_DESC.
pub const PLAGUE_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein unheilvoller Fund aus der Erweiterung „Pest der Toten\": Diese Zutat steht mit der Untotenplage über Himmelsrand in Verbindung und trägt die Spur der dunklen, zerstörerischen Magie, die die Toten aus ihren Gräbern erhebt."),
    ("en", "A sinister find from the Plague of the Dead add-on: this ingredient is bound to the undead scourge upon Skyrim, bearing the mark of the dark, destructive magic that raises the dead from their graves."),
    ("es", "Un hallazgo siniestro de la ampliación «Plaga de los Muertos»: este ingrediente está ligado al azote de no-muertos que asoló Skyrim y conserva la marca de la magia oscura y destructiva que levanta a los muertos de sus tumbas."),
    ("fr", "Une trouvaille sinistre de l'extension « Peste des morts » : cet ingrédient est lié au fléau des morts-vivants qui s'est abattu sur Bordeciel, et porte la marque de la magie sombre et destructrice qui relève les morts de leurs tombes."),
    ("it", "Un sinistro reperto del componente aggiuntivo «Peste dei Morti»: questo ingrediente è legato al flagello dei non-morti abbattutosi su Skyrim e reca il segno della magia oscura e distruttrice che risveglia i morti dalle loro tombe."),
    ("ja", "「死者の疫病」拡張がもたらした不吉な品——この素材はスカイリムを襲ったアンデッドの災厄に関わり、死者を墓から蘇らせる暗く破壊的な魔法の痕跡を宿している。"),
    ("pl", "Złowieszcze znalezisko z dodatku „Zaraza Umarłych\": ten składnik związany jest z plagą nieumarłych, która nawiedziła Skyrim, i nosi na sobie piętno mrocznej, niszczycielskiej magii wskrzeszającej zmarłych z grobów."),
    ("zh-Hant", "來自「死者瘟疫」擴充內容的不祥發現:這件材料與席捲天際的不死禍害有關,承載著那將死者從墳墓中喚醒的黑暗毀滅魔法之印記。"),
];
