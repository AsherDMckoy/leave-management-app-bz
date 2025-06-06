document.addEventListener("htmx:afterSettle", function(evt) {
    if (evt.detail.elt.getAttribute('data-background-request') === 'true') {
        return;
    }
    const isIndex = window.location.pathname === "/";
    const isRequests = window.location.pathname === "/requests";
    if (isIndex) {
	initCalendar();
	
    }
    if (isRequests) {
	initRequestsCalendar();
    }
});

function initCalendar() {
    const date = new Date();
    const today = date.toISOString().split("T")[0];
    const localDate = date.getFullYear() + "-" + 
	  String(date.getMonth() + 1).padStart(2, "0") + "-" + 
	  String(date.getDate()).padStart(2, "0");
    document.getElementById("start_date").setAttribute("min", localDate);
    document.getElementById("end_date").setAttribute("min", localDate);

    const startDateInput = document.getElementById("start_date");
    const endDateInput = document.getElementById("end_date");
    const daysDisplay = document.getElementById("leave_days");
    const daysPayload = document.getElementById("leave_days_payload");
    const clearDatesBtn = document.getElementById("clear-dates");
    const prevMonthBtn = document.getElementById("prev-month");
    const nextMonthBtn = document.getElementById("next-month");
    const monthYearDisplay = document.getElementById("current-month-year");
    const calendarBody = document.getElementById("calendar-body");
    const currentDate = new Date();

    let selectedDates = []; // Store selected dates

    function toggleClearButtonVisibility() {
	if (startDateInput.value || endDateInput.value) {
            clearDatesBtn.style.display = "block";
	} else {
            clearDatesBtn.style.display = "none";
	}
    }

    // Clear button event listener to reset the date selections
    clearDatesBtn.addEventListener("click", () => {
	resetSelection();
	startDateInput.min = localDate;
	endDateInput.min = localDate
	// Hide the clear button after clearing
	toggleClearButtonVisibility();
    });

    function validateLeaves() {
	const leave_days = parseFloat(document.getElementById('leave_days').innerText);
	const available_days = parseFloat(document.getElementById('available-leaves').innerText);
	
	if (available_days < leave_days) {
            console.log("insufficient available leave days");
            return false;
	}
	return true;
    }

    function generateCalendar() {
	const year = currentDate.getFullYear();
	const month = currentDate.getMonth();
	monthYearDisplay.textContent = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric' }).format(currentDate);

	const firstDay = new Date(year, month, 1).getDay();
	const daysInMonth = new Date(year, month + 1, 0).getDate();
	
	const date = new Date();
	const universalDate = date.toISOString().split("T")[0];
	const currentDay = date.getFullYear() + "-" + 
              String(date.getMonth() + 1).padStart(2, "0") + "-" + 
              String(date.getDate()).padStart(2, "0");

	let calendarHTML = "";
	let dayCounter = 1;

	for (let row = 0; row < 6; row++) {
            let rowHTML = "<tr>";
            for (let col = 0; col < 7; col++) {
		if (row === 0 && col < firstDay) {
                    rowHTML += "<td></td>"; // Empty cells before the first day
		} else if (dayCounter <= daysInMonth) {
                    const formattedDate = `${year}-${String(month + 1).padStart(2, '0')}-${String(dayCounter).padStart(2, '0')}`;
                    const isPastDate = formattedDate < currentDay;
		    const weekDay = new Date(year, month, dayCounter).getDay();
		    const isWeekend = (weekDay === 0 || weekDay === 6);

                    rowHTML += `<td class="calendar-day ${isPastDate ? 'disabled' : ''}" 
                                 data-date="${formattedDate}" ${isPastDate ? 'style="pointer-events:none;"' : ''} ${isWeekend ? 'style="color:red;"': ''}>
                                 ${dayCounter}
                                </td>`;
                    dayCounter++;
		} else {
                    rowHTML += "<td></td>"; // Empty cells after the last day
		}
            }
            rowHTML += "</tr>";
            calendarHTML += rowHTML;
            if (dayCounter > daysInMonth) break;
	}

	calendarBody.innerHTML = calendarHTML;
	highlightDates();
    }

    function changeMonth(offset) {
	currentDate.setDate(1);
	currentDate.setMonth(currentDate.getMonth() + offset);
	generateCalendar();
    }

    function highlightHelper(day, date, start, end){
	if(isNaN(start) && isNaN(end)){
	    day.classList.remove("highlight");
	}
	else if(!isNaN(start) && isNaN(end)) {
	    if(date.getTime() === start.getTime()) {
		day.classList.add("highlight");
	    } else {
		day.classList.remove("highlight");

	    }
	} else if(isNaN(start) && !isNaN(end)) {
	    if(date.getTime() === end.getTime()) {
		day.classList.add("highlight");
	    } else {
		day.classList.remove("highlight");
	    }
	} else {
	    if(date.getTime() >= start.getTime() && date.getTime() <= end.getTime()){
		day.classList.add("highlight");
	    }
	    else {
		day.classList.remove("highlight");
	    }
	}
    }

    function highlightDates() {
	let start = new Date(startDateInput.value);
	let end = new Date(endDateInput.value);

	document.querySelectorAll(".calendar-day").forEach(day => {
            let newDate = new Date(day.dataset.date);
	    if(!day.classList.contains("disabled")){
		highlightHelper(day, newDate, start, end);
	    }
	});
    }



    function isWeekend(date) {
	const day = date.getDay();
	return day === 0 || day === 6; // Sunday (0) & Saturday (6)
    }

    function parseLocalDate(dateString) {
	const [year, month, day] = dateString.split("-").map(Number);
	return new Date(year, month - 1, day); // Months are 0-based in JS Date
    }

    function updateDaysCount() {
	if (selectedDates.length === 2) {
            let start = parseLocalDate(selectedDates[0]);
            let end = parseLocalDate(selectedDates[1]);
            let count = 0;

            while (start <= end) {
		if (!isWeekend(start)) count++; // Exclude weekends
		start.setDate(start.getDate() + 1);
            }
            daysDisplay.textContent = count;
	    daysPayload.value = count;
	} else {
            daysDisplay.textContent = 0;
	    daysPayload.value = 0;
	}
    }

    function resetSelection() {
	selectedDates = [];
	startDateInput.value = "";
	endDateInput.value = "";
	daysDisplay.textContent = 0;
	daysPayload.value = 0;
	document.querySelectorAll(".calendar-day").forEach(day => day.classList.remove("selected", "highlight"));
    }

    function handleDateClick(event) {
	if (!event.target.classList.contains("calendar-day") || event.target.classList.contains("disabled")) {
            return;
	}

	let selectedDate = event.target.dataset.date;

	if (selectedDates.length === 0) {
            selectedDates.push(selectedDate);
            startDateInput.value = selectedDate;
            event.target.classList.add("selected");
	    toggleClearButtonVisibility()
	} else if (selectedDates.length === 1) {
            let firstDate = parseLocalDate(selectedDates[0]);
            let secondDate = parseLocalDate(selectedDate);

            if (secondDate < firstDate) {
		// Swap start and end dates if the second date is before the first
		startDateInput.value = selectedDate;
		endDateInput.value = selectedDates[0];
		selectedDates = [startDateInput.value, endDateInput.value];
            } else {
		startDateInput.value = selectedDates[0];
		endDateInput.value = selectedDate;
		selectedDates.push(selectedDate);
	    }
            document.querySelectorAll(".calendar-day").forEach(day => {
		let dayDate = parseLocalDate(day.dataset.date);
		if (dayDate >= firstDate && dayDate <= secondDate && !isWeekend(dayDate)) {
                    day.classList.add("selected");
		}
            });

            updateDaysCount();
	} else {
            resetSelection();
	    toggleClearButtonVisibility();
	}

	highlightDates();
	validateLeaves();
    }

    calendarBody.addEventListener("click", handleDateClick);

    prevMonthBtn.addEventListener("click", () => changeMonth(-1));
    nextMonthBtn.addEventListener("click", () => changeMonth(1));

    generateCalendar();   


    startDateInput.addEventListener("change", () => {
	const selectedDate = startDateInput.value;
	if(selectedDates.length === 0) {
	    selectedDates.push(selectedDate);
	} else {
	    selectedDates[0] = selectedDate;
	}
	if(selectedDate){
	    endDateInput.min = selectedDate;
	} else {
	    endDateInput.min = localDate;
	}
	if (endDateInput.value && endDateInput.value < selectedDate) {
            endDateInput.value = selectedDate; // Reset if invalid
	    selectedDates = [selectedDate, selectedDate];
	}
	validateLeaves();
	updateDaysCount();
	highlightDates();
	toggleClearButtonVisibility();
    });

    endDateInput.addEventListener("change", () => {
	if(selectedDates.length === 0){
	    selectedDates.push(endDateInput.value);
	} else {
	    selectedDates[1] = endDateInput.value;
	}
	validateLeaves();
	updateDaysCount();
	highlightDates();
	toggleClearButtonVisibility();
    });
}

function initRequestsCalendar() {
    const currentDate = new Date();
    const requestCards = document.querySelectorAll('.request_card');
    const prevMonthBtn = document.getElementById("prev-month");
    const nextMonthBtn = document.getElementById("next-month");
    const requestRows = document.querySelectorAll("#request-list-row");
    let selectedDates = [];

    function updateCalendar(dateString) {
	currentDate.setTime(new Date(dateString).getTime());
	generateRequestsCalendar(currentDate);
    }

    function highlightDates() {
	if(selectedDates.length === 0){
	    return;
	}
	const startDate = selectedDates[0];
	const endDate = selectedDates[1];
	document.querySelectorAll('.calendar-day').forEach(day => {
	    const date = new Date(day.dataset.date).toISOString().split("T")[0];
	    if((date >= startDate && date <= endDate)) {
		day.classList.add("highlight");
	    }
	});
    }
    function generateRequestsCalendar(currentDate) {
        const calendarBody = document.getElementById('requests-calendar-body');
	console.log("Requests calendar underway");
	const monthYearDisplay = document.getElementById('current-month-year');
	const year = currentDate.getFullYear();
	const month = currentDate.getMonth();
	monthYearDisplay.textContent = new Intl.DateTimeFormat('en-US', { month: 'long', year: 'numeric' }).format(currentDate);

	const firstDay = new Date(year, month, 1).getDay();
	const daysInMonth = new Date(year, month + 1, 0).getDate();
	
	const date = new Date();
	const universalDate = date.toISOString().split("T")[0];
	const currentDay = date.getFullYear() + "-" + 
              String(date.getMonth() + 1).padStart(2, "0") + "-" + 
              String(date.getDate()).padStart(2, "0");

	let calendarHTML = "";
	let dayCounter = 1;

	for (let row = 0; row < 6; row++) {
            let rowHTML = "<tr>";
            for (let col = 0; col < 7; col++) {
		if (row === 0 && col < firstDay) {
                    rowHTML += "<td></td>"; // Empty cells before the first day
		} else if (dayCounter <= daysInMonth) {
                    const formattedDate = `${year}-${String(month + 1).padStart(2, '0')}-${String(dayCounter).padStart(2, '0')}`;
		    const weekDay = new Date(year, month, dayCounter).getDay();
		    const isWeekend = (weekDay === 0 || weekDay === 6);
		    const baseStyle = "pointer-events:none;";
		    const weekendStyle = "color:red;";
		    const finalStyle = isWeekend ? `${baseStyle} ${weekendStyle}` : baseStyle;
                    rowHTML += `<td class="calendar-day" data-date="${formattedDate}" style="${finalStyle}">
                                 ${dayCounter}
                                </td>`;
                    dayCounter++;
		} else {
                    rowHTML += "<td></td>"; // Empty cells after the last day
		}
            }
            rowHTML += "</tr>";
            calendarHTML += rowHTML;
            if (dayCounter > daysInMonth) break;
	}

	calendarBody.innerHTML = calendarHTML;
	highlightDates();
    }
    generateRequestsCalendar(currentDate);

    function changeMonth(offset) {
	currentDate.setMonth(currentDate.getMonth() + offset);
	generateRequestsCalendar(currentDate);
    }

    function clearCalendar() {
	selectedDates = [];
	document.querySelectorAll(".calendar-day").forEach(day => day.classList.remove("selected", "highlight"));
    }

    if(requestRows.length != 0){
	requestRows.forEach(row => {
	    row.addEventListener("click", function(e) {
		clearCalendar();
		const startDateElem = row.querySelector("[data-start-date]");
		const endDateElem = row.querySelector("[data-end-date]");
		const startDate = startDateElem?.dataset.startDate;
		const endDate = endDateElem?.dataset.endDate;

		selectedDates = [startDate, endDate]

		updateCalendar(startDate);
		highlightDates();
	    });
	});
    }

    const table = document.querySelector(".my_leaves_list__table");
    // if (table) {
    // 	date = new Date();
    // 	table.addEventListener('mouseleave', () => generateRequestsCalendar());
    // }

    document.querySelectorAll(".request_card").forEach(card => {
	card.addEventListener("click", function (e) {
            if (e.target.closest("button")) return;
            // Get the start and end date from within the card
            const startDateElem = card.querySelector("[data-start-date]");
            const endDateElem = card.querySelector("[data-end-date]");

            const startDate = startDateElem?.dataset.startDate;
            const endDate = endDateElem?.dataset.endDate;

	    selectedDates = [startDate, endDate]

	    updateCalendar(startDate);
	    highlightDates();
	});
    });
    prevMonthBtn.addEventListener("click", () => changeMonth(-1));
    nextMonthBtn.addEventListener("click", () => changeMonth(1));
}

