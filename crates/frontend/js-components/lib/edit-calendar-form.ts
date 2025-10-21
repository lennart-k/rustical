import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { escapeXml } from ".";

@customElement("edit-calendar-form")
export class EditCalendarForm extends LitElement {
  constructor() {
    super()
  }

  protected override createRenderRoot() {
    return this
  }

  @property()
  principal: string
  @property()
  cal_id: string

  @property()
  displayname: string = ''
  @property()
  description: string = ''
  @property()
  timezone_id: string = ''
  @property()
  color: string = ''
  @property({
    converter: {
      fromAttribute: (value, _type) => new Set(value ? JSON.parse(value) : []),
      toAttribute: (value, _type) => JSON.stringify(value)
    }
  })
  components: Set<"VEVENT" | "VTODO" | "VJOURNAL"> = new Set()

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()


  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${ref(this.dialog)}>
        <h3>Edit calendar</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" .value=${this.displayname} @change=${e => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <input type="text" list="timezone-list" name="timezone" .value=${this.timezone_id} @change=${e => this.timezone_id = e.target.value} />
            <datalist id="timezone-list">
              <option>Asia/Irkutsk</option>
              <option>Asia/Anadyr</option>
              <option>PRC</option>
              <option>Africa/Maputo</option>
              <option>America/Indiana/Vevay</option>
              <option>Pacific/Saipan</option>
              <option>Australia/Victoria</option>
              <option>Atlantic/Madeira</option>
              <option>America/Iqaluit</option>
              <option>Europe/Zagreb</option>
              <option>Africa/Luanda</option>
              <option>America/Adak</option>
              <option>America/Moncton</option>
              <option>Pacific/Easter</option>
              <option>Africa/Lome</option>
              <option>Europe/Minsk</option>
              <option>Asia/Almaty</option>
              <option>America/Toronto</option>
              <option>Etc/Zulu</option>
              <option>EET</option>
              <option>Asia/Khandyga</option>
              <option>MET</option>
              <option>America/Marigot</option>
              <option>America/Jamaica</option>
              <option>Europe/Riga</option>
              <option>Asia/Rangoon</option>
              <option>Asia/Karachi</option>
              <option>Singapore</option>
              <option>Etc/GMT0</option>
              <option>Australia/ACT</option>
              <option>Zulu</option>
              <option>Canada/Newfoundland</option>
              <option>Antarctica/South_Pole</option>
              <option>Asia/Ashkhabad</option>
              <option>Asia/Kolkata</option>
              <option>America/Thunder_Bay</option>
              <option>Etc/GMT+2</option>
              <option>Asia/Krasnoyarsk</option>
              <option>Canada/Saskatchewan</option>
              <option>Europe/Sofia</option>
              <option>Europe/Kirov</option>
              <option>Africa/Dar_es_Salaam</option>
              <option>America/Los_Angeles</option>
              <option>America/Catamarca</option>
              <option>Africa/Ndjamena</option>
              <option>Pacific/Noumea</option>
              <option>America/Bahia_Banderas</option>
              <option>America/Matamoros</option>
              <option>Africa/Kampala</option>
              <option>Canada/Eastern</option>
              <option>Africa/Maseru</option>
              <option>America/Belize</option>
              <option>Asia/Katmandu</option>
              <option>Etc/UTC</option>
              <option>Europe/Belgrade</option>
              <option>America/Goose_Bay</option>
              <option>Etc/GMT+0</option>
              <option>Pacific/Pohnpei</option>
              <option>America/Paramaribo</option>
              <option>Indian/Antananarivo</option>
              <option>Asia/Omsk</option>
              <option>Pacific/Honolulu</option>
              <option>Atlantic/Stanley</option>
              <option>Asia/Jakarta</option>
              <option>America/Inuvik</option>
              <option>Africa/Monrovia</option>
              <option>Australia/Lord_Howe</option>
              <option>America/Havana</option>
              <option>America/Miquelon</option>
              <option>Indian/Kerguelen</option>
              <option>Iran</option>
              <option>Asia/Urumqi</option>
              <option>Etc/GMT+7</option>
              <option>Asia/Tehran</option>
              <option>America/Guyana</option>
              <option>Australia/Tasmania</option>
              <option>Pacific/Samoa</option>
              <option>Europe/Vilnius</option>
              <option>America/North_Dakota/Beulah</option>
              <option>Pacific/Enderbury</option>
              <option>Etc/GMT-0</option>
              <option>Arctic/Longyearbyen</option>
              <option>America/Hermosillo</option>
              <option>Antarctica/McMurdo</option>
              <option>Asia/Kuala_Lumpur</option>
              <option>Pacific/Wake</option>
              <option>America/Grand_Turk</option>
              <option>America/Ojinaga</option>
              <option>US/Central</option>
              <option>America/Montreal</option>
              <option>America/St_Kitts</option>
              <option>Asia/Hebron</option>
              <option>Europe/Athens</option>
              <option>America/Winnipeg</option>
              <option>America/Virgin</option>
              <option>America/Sao_Paulo</option>
              <option>Asia/Kabul</option>
              <option>Etc/GMT+3</option>
              <option>America/Punta_Arenas</option>
              <option>Africa/Mogadishu</option>
              <option>America/Indiana/Petersburg</option>
              <option>Asia/Barnaul</option>
              <option>America/Juneau</option>
              <option>Europe/Luxembourg</option>
              <option>America/Dominica</option>
              <option>Europe/Guernsey</option>
              <option>America/Anchorage</option>
              <option>America/Barbados</option>
              <option>Pacific/Midway</option>
              <option>Asia/Atyrau</option>
              <option>Pacific/Yap</option>
              <option>Etc/Universal</option>
              <option>America/Yakutat</option>
              <option>America/Coral_Harbour</option>
              <option>Atlantic/Reykjavik</option>
              <option>America/Noronha</option>
              <option>Europe/Stockholm</option>
              <option>Africa/Casablanca</option>
              <option>Africa/Khartoum</option>
              <option>Australia/Broken_Hill</option>
              <option>America/Fort_Wayne</option>
              <option>Africa/Djibouti</option>
              <option>America/Danmarkshavn</option>
              <option>Australia/Sydney</option>
              <option>America/Ensenada</option>
              <option>Europe/Kaliningrad</option>
              <option>Africa/Dakar</option>
              <option>Asia/Muscat</option>
              <option>Asia/Baku</option>
              <option>Atlantic/St_Helena</option>
              <option>Pacific/Tarawa</option>
              <option>Asia/Brunei</option>
              <option>America/Asuncion</option>
              <option>America/Caracas</option>
              <option>America/Montevideo</option>
              <option>Europe/Busingen</option>
              <option>Europe/Nicosia</option>
              <option>America/Coyhaique</option>
              <option>Pacific/Marquesas</option>
              <option>America/Santiago</option>
              <option>America/Shiprock</option>
              <option>Etc/GMT+11</option>
              <option>Africa/Conakry</option>
              <option>Asia/Riyadh</option>
              <option>Europe/Brussels</option>
              <option>Portugal</option>
              <option>America/St_Johns</option>
              <option>America/Mexico_City</option>
              <option>Brazil/West</option>
              <option>Africa/Harare</option>
              <option>Indian/Christmas</option>
              <option>America/Knox_IN</option>
              <option>America/Nuuk</option>
              <option>Antarctica/Casey</option>
              <option>Etc/GMT-6</option>
              <option>America/Indiana/Tell_City</option>
              <option>ROK</option>
              <option>Indian/Maldives</option>
              <option>America/Chicago</option>
              <option>Asia/Magadan</option>
              <option>Pacific/Palau</option>
              <option>Europe/Zaporozhye</option>
              <option>Pacific/Johnston</option>
              <option>Asia/Samarkand</option>
              <option>Iceland</option>
              <option>Europe/London</option>
              <option>America/Santo_Domingo</option>
              <option>Chile/EasterIsland</option>
              <option>America/Godthab</option>
              <option>America/Argentina/Buenos_Aires</option>
              <option>Asia/Chita</option>
              <option>Asia/Jerusalem</option>
              <option>US/Alaska</option>
              <option>Etc/GMT-8</option>
              <option>Australia/Currie</option>
              <option>Africa/Accra</option>
              <option>Libya</option>
              <option>Europe/Zurich</option>
              <option>Etc/GMT</option>
              <option>Etc/GMT-12</option>
              <option>Etc/GMT-4</option>
              <option>Asia/Shanghai</option>
              <option>Europe/Copenhagen</option>
              <option>America/Cuiaba</option>
              <option>Europe/Vatican</option>
              <option>America/Creston</option>
              <option>Asia/Ashgabat</option>
              <option>Asia/Yakutsk</option>
              <option>America/Swift_Current</option>
              <option>America/Tijuana</option>
              <option>America/Boise</option>
              <option>Asia/Famagusta</option>
              <option>Europe/Simferopol</option>
              <option>America/Sitka</option>
              <option>America/Port-au-Prince</option>
              <option>Mexico/BajaNorte</option>
              <option>America/Fortaleza</option>
              <option>Antarctica/Davis</option>
              <option>America/Santa_Isabel</option>
              <option>Africa/Juba</option>
              <option>Etc/Greenwich</option>
              <option>Pacific/Wallis</option>
              <option>Etc/GMT+6</option>
              <option>Asia/Chungking</option>
              <option>Antarctica/DumontDUrville</option>
              <option>Pacific/Fiji</option>
              <option>Europe/San_Marino</option>
              <option>America/Resolute</option>
              <option>Pacific/Kiritimati</option>
              <option>Asia/Pyongyang</option>
              <option>GMT-0</option>
              <option>NZ-CHAT</option>
              <option>America/Argentina/La_Rioja</option>
              <option>America/Montserrat</option>
              <option>Africa/Douala</option>
              <option>Europe/Malta</option>
              <option>America/Indiana/Marengo</option>
              <option>America/Tortola</option>
              <option>America/Tegucigalpa</option>
              <option>America/Phoenix</option>
              <option>America/Cambridge_Bay</option>
              <option>America/St_Barthelemy</option>
              <option>Asia/Dhaka</option>
              <option>Asia/Taipei</option>
              <option>UCT</option>
              <option>Africa/Gaborone</option>
              <option>Atlantic/Bermuda</option>
              <option>America/Yellowknife</option>
              <option>Antarctica/Mawson</option>
              <option>Navajo</option>
              <option>Africa/Tripoli</option>
              <option>America/Atikokan</option>
              <option>America/Monterrey</option>
              <option>America/Blanc-Sablon</option>
              <option>Etc/GMT-7</option>
              <option>Africa/Bangui</option>
              <option>America/Bahia</option>
              <option>America/Denver</option>
              <option>America/Maceio</option>
              <option>Pacific/Niue</option>
              <option>Asia/Vientiane</option>
              <option>Asia/Aden</option>
              <option>Asia/Makassar</option>
              <option>America/Managua</option>
              <option>America/Whitehorse</option>
              <option>EST5EDT</option>
              <option>Indian/Mauritius</option>
              <option>Europe/Tirane</option>
              <option>Europe/Ljubljana</option>
              <option>Europe/Mariehamn</option>
              <option>Pacific/Guam</option>
              <option>Australia/Canberra</option>
              <option>America/Porto_Velho</option>
              <option>Pacific/Tongatapu</option>
              <option>America/Rainy_River</option>
              <option>MST</option>
              <option>America/Kralendijk</option>
              <option>W-SU</option>
              <option>Asia/Bangkok</option>
              <option>Indian/Mayotte</option>
              <option>Asia/Aqtau</option>
              <option>America/Costa_Rica</option>
              <option>GB-Eire</option>
              <option>Asia/Kathmandu</option>
              <option>Asia/Macao</option>
              <option>America/Guayaquil</option>
              <option>America/Nome</option>
              <option>Asia/Thimphu</option>
              <option>Etc/GMT+8</option>
              <option>America/Belem</option>
              <option>Asia/Aqtobe</option>
              <option>Pacific/Pitcairn</option>
              <option>Asia/Ulan_Bator</option>
              <option>Etc/GMT+12</option>
              <option>Europe/Budapest</option>
              <option>America/Guatemala</option>
              <option>Asia/Tokyo</option>
              <option>America/Atka</option>
              <option>Brazil/DeNoronha</option>
              <option>Australia/North</option>
              <option>Universal</option>
              <option>Canada/Pacific</option>
              <option>Asia/Novosibirsk</option>
              <option>ROC</option>
              <option>America/Curacao</option>
              <option>Asia/Ujung_Pandang</option>
              <option>Asia/Ust-Nera</option>
              <option>Pacific/Majuro</option>
              <option>Pacific/Kosrae</option>
              <option>Africa/Cairo</option>
              <option>Asia/Istanbul</option>
              <option>Asia/Yangon</option>
              <option>Japan</option>
              <option>Africa/Porto-Novo</option>
              <option>Africa/Ouagadougou</option>
              <option>Pacific/Truk</option>
              <option>Australia/Perth</option>
              <option>Asia/Singapore</option>
              <option>US/Aleutian</option>
              <option>Asia/Seoul</option>
              <option>America/Regina</option>
              <option>Europe/Saratov</option>
              <option>Pacific/Efate</option>
              <option>Antarctica/Macquarie</option>
              <option>Atlantic/Faeroe</option>
              <option>Asia/Damascus</option>
              <option>Asia/Vladivostok</option>
              <option>Asia/Gaza</option>
              <option>Africa/El_Aaiun</option>
              <option>America/Port_of_Spain</option>
              <option>America/Argentina/Mendoza</option>
              <option>America/Jujuy</option>
              <option>Pacific/Kwajalein</option>
              <option>Asia/Manila</option>
              <option>Etc/GMT+9</option>
              <option>Europe/Berlin</option>
              <option>America/Rio_Branco</option>
              <option>America/St_Vincent</option>
              <option>Etc/GMT+5</option>
              <option>Africa/Kinshasa</option>
              <option>America/Argentina/Catamarca</option>
              <option>Asia/Baghdad</option>
              <option>Etc/GMT-1</option>
              <option>Europe/Gibraltar</option>
              <option>Africa/Banjul</option>
              <option>America/Argentina/ComodRivadavia</option>
              <option>Europe/Sarajevo</option>
              <option>America/Indiana/Indianapolis</option>
              <option>Europe/Kiev</option>
              <option>US/Pacific</option>
              <option>America/Metlakatla</option>
              <option>Europe/Uzhgorod</option>
              <option>Asia/Hovd</option>
              <option>Etc/UCT</option>
              <option>Australia/Darwin</option>
              <option>America/North_Dakota/Center</option>
              <option>Asia/Dacca</option>
              <option>Atlantic/Canary</option>
              <option>Pacific/Guadalcanal</option>
              <option>Asia/Macau</option>
              <option>Europe/Tallinn</option>
              <option>Africa/Blantyre</option>
              <option>Africa/Timbuktu</option>
              <option>Pacific/Galapagos</option>
              <option>America/Boa_Vista</option>
              <option>America/El_Salvador</option>
              <option>Australia/Brisbane</option>
              <option>America/Puerto_Rico</option>
              <option>Africa/Niamey</option>
              <option>Europe/Andorra</option>
              <option>Antarctica/Vostok</option>
              <option>Asia/Thimbu</option>
              <option>Africa/Asmera</option>
              <option>America/Indiana/Winamac</option>
              <option>MST7MDT</option>
              <option>America/St_Thomas</option>
              <option>CET</option>
              <option>Canada/Mountain</option>
              <option>America/Santarem</option>
              <option>Antarctica/Syowa</option>
              <option>Europe/Bucharest</option>
              <option>Europe/Vaduz</option>
              <option>Europe/Tiraspol</option>
              <option>Europe/Istanbul</option>
              <option>Asia/Saigon</option>
              <option>UTC</option>
              <option>Turkey</option>
              <option>America/Edmonton</option>
              <option>Etc/GMT-9</option>
              <option>Hongkong</option>
              <option>Indian/Mahe</option>
              <option>Asia/Kamchatka</option>
              <option>Africa/Nairobi</option>
              <option>Europe/Samara</option>
              <option>Pacific/Chatham</option>
              <option>America/Guadeloupe</option>
              <option>Europe/Warsaw</option>
              <option>America/New_York</option>
              <option>Africa/Windhoek</option>
              <option>America/Panama</option>
              <option>America/Argentina/Ushuaia</option>
              <option>Asia/Sakhalin</option>
              <option>America/La_Paz</option>
              <option>US/Arizona</option>
              <option>US/Eastern</option>
              <option>Pacific/Fakaofo</option>
              <option>Asia/Kuwait</option>
              <option>America/Argentina/Salta</option>
              <option>Indian/Comoro</option>
              <option>Pacific/Funafuti</option>
              <option>Asia/Tel_Aviv</option>
              <option>Europe/Dublin</option>
              <option>America/Mendoza</option>
              <option>US/Mountain</option>
              <option>Pacific/Nauru</option>
              <option>America/Kentucky/Louisville</option>
              <option>Atlantic/Azores</option>
              <option>Etc/GMT-2</option>
              <option>Canada/Central</option>
              <option>Africa/Addis_Ababa</option>
              <option>Europe/Amsterdam</option>
              <option>Etc/GMT-11</option>
              <option>US/Hawaii</option>
              <option>Asia/Amman</option>
              <option>America/Manaus</option>
              <option>America/Menominee</option>
              <option>Asia/Yekaterinburg</option>
              <option>America/Araguaina</option>
              <option>Africa/Bujumbura</option>
              <option>Antarctica/Rothera</option>
              <option>Asia/Bishkek</option>
              <option>America/Grenada</option>
              <option>Asia/Calcutta</option>
              <option>America/Martinique</option>
              <option>Asia/Jayapura</option>
              <option>America/Cancun</option>
              <option>America/Argentina/Tucuman</option>
              <option>Asia/Chongqing</option>
              <option>America/Rankin_Inlet</option>
              <option>Indian/Cocos</option>
              <option>US/East-Indiana</option>
              <option>Etc/GMT+10</option>
              <option>EST</option>
              <option>America/Ciudad_Juarez</option>
              <option>Europe/Helsinki</option>
              <option>Etc/GMT-13</option>
              <option>America/Fort_Nelson</option>
              <option>US/Indiana-Starke</option>
              <option>Asia/Pontianak</option>
              <option>Pacific/Norfolk</option>
              <option>Africa/Bamako</option>
              <option>Australia/Eucla</option>
              <option>Australia/Queensland</option>
              <option>Europe/Prague</option>
              <option>Asia/Tbilisi</option>
              <option>Africa/Malabo</option>
              <option>Pacific/Apia</option>
              <option>Asia/Phnom_Penh</option>
              <option>Atlantic/Cape_Verde</option>
              <option>Asia/Dili</option>
              <option>Pacific/Pago_Pago</option>
              <option>America/Thule</option>
              <option>Pacific/Port_Moresby</option>
              <option>Africa/Kigali</option>
              <option>Canada/Yukon</option>
              <option>Asia/Colombo</option>
              <option>Jamaica</option>
              <option>America/Argentina/Cordoba</option>
              <option>Kwajalein</option>
              <option>Pacific/Gambier</option>
              <option>America/North_Dakota/New_Salem</option>
              <option>Etc/GMT-10</option>
              <option>Europe/Jersey</option>
              <option>Etc/GMT-3</option>
              <option>Etc/GMT+1</option>
              <option>America/Rosario</option>
              <option>Asia/Nicosia</option>
              <option>America/Lima</option>
              <option>Pacific/Ponape</option>
              <option>Africa/Libreville</option>
              <option>Europe/Skopje</option>
              <option>Europe/Paris</option>
              <option>Africa/Brazzaville</option>
              <option>Africa/Johannesburg</option>
              <option>America/Lower_Princes</option>
              <option>America/Argentina/Jujuy</option>
              <option>Asia/Kashgar</option>
              <option>US/Michigan</option>
              <option>America/Dawson_Creek</option>
              <option>America/Indiana/Vincennes</option>
              <option>Asia/Tashkent</option>
              <option>Africa/Freetown</option>
              <option>America/Anguilla</option>
              <option>America/Argentina/Rio_Gallegos</option>
              <option>NZ</option>
              <option>Atlantic/South_Georgia</option>
              <option>Europe/Isle_of_Man</option>
              <option>Africa/Ceuta</option>
              <option>Asia/Kuching</option>
              <option>Asia/Hong_Kong</option>
              <option>Australia/Hobart</option>
              <option>Africa/Asmara</option>
              <option>Europe/Rome</option>
              <option>Atlantic/Faroe</option>
              <option>America/Cordoba</option>
              <option>Pacific/Auckland</option>
              <option>Europe/Oslo</option>
              <option>Europe/Astrakhan</option>
              <option>Brazil/East</option>
              <option>Africa/Lagos</option>
              <option>Africa/Mbabane</option>
              <option>Brazil/Acre</option>
              <option>Indian/Chagos</option>
              <option>Europe/Podgorica</option>
              <option>Antarctica/Troll</option>
              <option>America/Porto_Acre</option>
              <option>America/Dawson</option>
              <option>America/Cayenne</option>
              <option>Africa/Bissau</option>
              <option>America/Nipigon</option>
              <option>Asia/Choibalsan</option>
              <option>America/Argentina/San_Luis</option>
              <option>Asia/Dubai</option>
              <option>Europe/Ulyanovsk</option>
              <option>Australia/Melbourne</option>
              <option>America/Cayman</option>
              <option>America/Mazatlan</option>
              <option>Cuba</option>
              <option>Egypt</option>
              <option>Africa/Tunis</option>
              <option>Asia/Ulaanbaatar</option>
              <option>Europe/Chisinau</option>
              <option>Australia/Lindeman</option>
              <option>Mexico/General</option>
              <option>Australia/West</option>
              <option>Asia/Tomsk</option>
              <option>Europe/Kyiv</option>
              <option>Europe/Volgograd</option>
              <option>America/Scoresbysund</option>
              <option>America/Pangnirtung</option>
              <option>Chile/Continental</option>
              <option>Asia/Qatar</option>
              <option>Asia/Qyzylorda</option>
              <option>Asia/Novokuznetsk</option>
              <option>Australia/LHI</option>
              <option>Europe/Belfast</option>
              <option>Mexico/BajaSur</option>
              <option>Poland</option>
              <option>Asia/Dushanbe</option>
              <option>Israel</option>
              <option>America/Argentina/San_Juan</option>
              <option>Eire</option>
              <option>Etc/GMT-5</option>
              <option>America/Eirunepe</option>
              <option>Greenwich</option>
              <option>America/Recife</option>
              <option>Antarctica/Palmer</option>
              <option>America/Bogota</option>
              <option>Australia/South</option>
              <option>America/Chihuahua</option>
              <option>Africa/Lubumbashi</option>
              <option>Asia/Bahrain</option>
              <option>Indian/Reunion</option>
              <option>Atlantic/Jan_Mayen</option>
              <option>Australia/Yancowinna</option>
              <option>Europe/Bratislava</option>
              <option>America/Glace_Bay</option>
              <option>Asia/Ho_Chi_Minh</option>
              <option>GB</option>
              <option>Europe/Lisbon</option>
              <option>Africa/Abidjan</option>
              <option>Europe/Monaco</option>
              <option>Africa/Algiers</option>
              <option>Canada/Atlantic</option>
              <option>Europe/Vienna</option>
              <option>America/Halifax</option>
              <option>America/Indianapolis</option>
              <option>America/Louisville</option>
              <option>America/St_Lucia</option>
              <option>America/Campo_Grande</option>
              <option>Australia/NSW</option>
              <option>Europe/Madrid</option>
              <option>America/Merida</option>
              <option>GMT0</option>
              <option>Pacific/Rarotonga</option>
              <option>Australia/Adelaide</option>
              <option>Africa/Nouakchott</option>
              <option>GMT</option>
              <option>Etc/GMT-14</option>
              <option>America/Vancouver</option>
              <option>America/Detroit</option>
              <option>America/Indiana/Knox</option>
              <option>Asia/Yerevan</option>
              <option>CST6CDT</option>
              <option>America/Antigua</option>
              <option>HST</option>
              <option>Pacific/Tahiti</option>
              <option>Pacific/Chuuk</option>
              <option>Asia/Srednekolymsk</option>
              <option>WET</option>
              <option>US/Samoa</option>
              <option>Pacific/Kanton</option>
              <option>Africa/Sao_Tome</option>
              <option>Asia/Oral</option>
              <option>America/Aruba</option>
              <option>America/Buenos_Aires</option>
              <option>Africa/Lusaka</option>
              <option>Asia/Qostanay</option>
              <option>America/Nassau</option>
              <option>Pacific/Bougainville</option>
              <option>Asia/Beirut</option>
              <option>Europe/Moscow</option>
              <option>PST8PDT</option>
              <option>Etc/GMT+4</option>
              <option>GMT+0</option>
              <option>America/Kentucky/Monticello</option>
              <option>Asia/Harbin</option>
            </datalist>
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" .value=${this.description} @change=${e => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" name="color" .value=${this.color} @change=${e => this.color = e.target.value} />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map(comp => html`
            <label>
              Support ${comp}
              <input type="checkbox" value=${comp} ?checked=${this.components.has(comp)} @change=${e => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} />
            </label>
            <br>
          `)}
          <br>
          <button type="submit">Submit</button>
          <button type="submit" @click=${event => { event.preventDefault(); this.dialog.value.close(); this.form.value.reset() }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
    if (!this.principal) {
      alert("Empty principal")
      return
    }
    if (!this.cal_id) {
      alert("Empty id")
      return
    }
    if (!this.displayname) {
      alert("Empty displayname")
      return
    }
    if (!this.components.size) {
      alert("No calendar components selected")
      return
    }
    let response = await fetch(`/caldav/principal/${this.principal}/${this.cal_id}`, {
      method: 'PROPPATCH',
      headers: {
        'Content-Type': 'application/xml'
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ''}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ''}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ''}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map(comp => `<CAL:comp name="${escapeXml(comp)}" />`).join('\n')}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.timezone_id ? `<CAL:calendar-timezone-id />` : ''}
            ${!this.description ? '<CAL:calendar-description />' : ''}
            ${!this.color ? '<ICAL:calendar-color />' : ''}
          </prop>
        </remove>
      </propertyupdate>
      `
    })

    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`)
      return null
    }

    window.location.reload()
    return null
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'edit-calendar-form': EditCalendarForm
  }
}
