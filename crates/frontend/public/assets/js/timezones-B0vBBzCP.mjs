let timezonesPromise = null;
async function getTimezones() {
  timezonesPromise ||= new Promise(async (resolve, reject) => {
    try {
      let response = await fetch("/frontend/_timezones.json");
      resolve(await response.json());
    } catch (e) {
      reject(e);
    }
  });
  return await timezonesPromise;
}
export {
  getTimezones as g
};
