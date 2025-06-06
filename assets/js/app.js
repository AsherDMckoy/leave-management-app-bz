document.addEventListener('DOMContentLoaded', function () {
    initializeTableFilters();
});
document.addEventListener("htmx:afterSettle", function (event) {
    updateLeaveDates();
    activateTab();
    initializeTableFilters();

    const is_profile = (window.location.pathname === "/profile");
    if (is_profile) {
        initializeProfile();
    }
});
    
function updateLeaveDates() {
  const teamPrevDateElements = document.querySelectorAll(".team_prev_details__dates");
  teamPrevDateElements.forEach(function (dateElement) {
    let startDate = dateElement.dataset.start;
    let endDate = dateElement.dataset.end;
    if (startDate && endDate) {
      dateElement.textContent = formatDate(startDate) + " - " + formatDate(endDate);
    }
  });

  const myPrevStartDateElements = document.querySelectorAll("#start-date.my_prev__details__date");
  const myPrevEndDateElements = document.querySelectorAll("#end-date.my_prev__details__date");
  if (myPrevStartDateElements && myPrevEndDateElements) {
    myPrevStartDateElements.forEach(function (startDateElement) { 
      startDateElement.textContent = formatDate(startDateElement.dataset.start);
    });
    myPrevEndDateElements.forEach(function (endDateElement) {
      endDateElement.textContent = formatDate(endDateElement.dataset.end);
    });
  }
}

function formatDate(isoDate) {
  // Split the ISO date string into components
  const [year, month, day] = isoDate.split('-').map(num => parseInt(num, 10));
  
  // Create date using local timezone (months are 0-indexed in JS)
  const date = new Date(year, month - 1, day);
  if (isNaN(date)) {
    console.error("Invalid date:", isoDate);
    return isoDate;
  }
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric"
  });
}

// Run initially for elements already in the DOM
updateLeaveDates();

function toggleNotifications(event) {
    let panel = document.getElementById("notification-panel");
    let backdrop = document.getElementById("notification-backdrop");
    
    if (panel.classList.contains("hidden")) {
        // Show backdrop first
        backdrop.classList.remove("hidden");
        setTimeout(() => {
            backdrop.classList.add("visible");
        }, 10);
        
        // Then show panel
        panel.classList.remove("hidden");
        setTimeout(() => {
            panel.classList.add("visible");
        }, 10);
    } else {
        // Hide panel first
        checkForNotifications();
        panel.classList.remove("visible");
        setTimeout(() => {
            panel.classList.add("hidden");
        }, 300);
        
        // Then hide backdrop
        backdrop.classList.remove("visible");
        setTimeout(() => {
            backdrop.classList.add("hidden");
        }, 300);
    }
}

function activateTab() {
    const path = window.location.pathname;
    let navs = document.querySelectorAll('.nav-item');
    let activeNav;
    navs.forEach(nav => {
	nav.classList.remove('active');
	nav.setAttribute('aria-selected', 'false');
	link = nav.closest('a').getAttribute('hx-get');
	if(link === path){
	    activeNav = nav;
	}
    });
    activeNav.classList.add('active');
    activeNav.setAttribute('aria-selected', 'false');
}

function activateRequestsSubheader() {
    const navs = document.querySelectorAll('#requests-nav');
    navs.forEach(nav => {
	nav.classList.remove('active');
	nav.setAttribute('aria-selected', 'false');
    });
    event.target.classList.add('active');
    event.target.setAttribute('aria-selected', 'true');
}

activateTab();

function checkForNotifications() {
  const notificationPanel = document.getElementById('notification-panel');
  const notificationButton = document.getElementById('notification-button');
  
  if (notificationPanel && notificationButton) {
    const hasNotifications = notificationPanel.querySelector('.notification') !== null;
    notificationButton.classList.toggle('has-notifications', hasNotifications);
  }
}

// Check for notifications when the panel loads
document.body.addEventListener('htmx:afterSwap', function(evt) {
  if (evt.detail.target.id === 'notification-panel') {
    checkForNotifications();
  }
});

// Check for notifications periodically
setInterval(checkForNotifications, 30000); // Check every 30 seconds

function initializeTableFilters() {
  // Get all filter elements
  const nameSearch = document.getElementById('name-search');
  const commentSearch = document.getElementById('comment-search');
  const startDateFilter = document.getElementById('start-date-filter');
  const endDateFilter = document.getElementById('end-date-filter');
  const leaveTypeFilter = document.getElementById('leave-type-filter');
  const statusFilter = document.getElementById('status-filter');
  const clearFiltersBtn = document.getElementById('clear-filters');
  
  if (!nameSearch && !commentSearch) return; // Exit if we're not on a filtered page
  
  const updateClearFiltersButton = () => {
    let hasFilters = false;
    
    if (nameSearch) {
      hasFilters = hasFilters || nameSearch.value;
    }
    if (commentSearch) {
      hasFilters = hasFilters || commentSearch.value;
    }
    if (startDateFilter) {
      hasFilters = hasFilters || startDateFilter.value;
    }
    if (endDateFilter) {
      hasFilters = hasFilters || endDateFilter.value;
    }
    if (leaveTypeFilter) {
      hasFilters = hasFilters || leaveTypeFilter.value;
    }
    if (statusFilter) {
      hasFilters = hasFilters || statusFilter.value;
    }
    
    clearFiltersBtn.classList.toggle('visible', hasFilters);
  };
  
  const clearFilters = () => {
    if (nameSearch) nameSearch.value = '';
    if (commentSearch) commentSearch.value = '';
    if (startDateFilter) startDateFilter.value = '';
    if (endDateFilter) endDateFilter.value = '';
    if (leaveTypeFilter) leaveTypeFilter.value = '';
    if (statusFilter) statusFilter.value = '';
    filterTable();
    updateClearFiltersButton();
  };
  
  const filterTable = () => {
    // Determine which table we're filtering
    const table = document.querySelector('.my_leaves_list__table') || document.querySelector('.requests_lists__table');
    if (!table) return;
    
    const rows = table.querySelectorAll('tbody tr');
    const isLeavesList = table.classList.contains('my_leaves_list__table');
    
    rows.forEach(row => {
      let showRow = true;
      
      // Handle comment search for leaves list
      if (isLeavesList && commentSearch) {
        const comment = row.querySelector('td:nth-child(6)').textContent.toLowerCase();
        const searchTerm = commentSearch.value.toLowerCase();
        if (searchTerm && !comment.includes(searchTerm)) {
          showRow = false;
        }
      }
      
      // Handle name search for requests list
      if (!isLeavesList && nameSearch) {
        const name = row.querySelector('td:nth-child(1)').textContent.toLowerCase();
        const searchTerm = nameSearch.value.toLowerCase();
        if (searchTerm && !name.includes(searchTerm)) {
          showRow = false;
        }
      }
      
      // Handle date filters
      if (startDateFilter && startDateFilter.value) {
        const startDate = new Date(startDateFilter.value);
        const rowStartDate = new Date(row.querySelector('td:nth-child(1)').getAttribute('data-start-date'));
        if (rowStartDate < startDate) {
          showRow = false;
        }
      }
      
      if (endDateFilter && endDateFilter.value) {
        const endDate = new Date(endDateFilter.value);
        const rowEndDate = new Date(row.querySelector('td:nth-child(2)').getAttribute('data-end-date'));
        if (rowEndDate > endDate) {
          showRow = false;
        }
      }
      
      // Handle leave type filter for requests list
      if (!isLeavesList && leaveTypeFilter && leaveTypeFilter.value) {
        const leaveType = row.querySelector('td:nth-child(4)').textContent;
        if (leaveTypeFilter.value !== leaveType) {
          showRow = false;
        }
      }
      
      // Handle status filter for leaves list
      if (isLeavesList && statusFilter && statusFilter.value) {
        const status = row.querySelector('td:nth-child(5)').textContent.trim();
        if (statusFilter.value !== status) {
          showRow = false;
        }
      }
      
      row.style.display = showRow ? '' : 'none';
    });
    
    updateClearFiltersButton();
  };
  
  // Add event listeners
  if (nameSearch) nameSearch.addEventListener('input', filterTable);
  if (commentSearch) commentSearch.addEventListener('input', filterTable);
  if (startDateFilter) startDateFilter.addEventListener('change', filterTable);
  if (endDateFilter) endDateFilter.addEventListener('change', filterTable);
  if (leaveTypeFilter) leaveTypeFilter.addEventListener('change', filterTable);
  if (statusFilter) statusFilter.addEventListener('change', filterTable);
  if (clearFiltersBtn) clearFiltersBtn.addEventListener('click', clearFilters);
  
  // Initial filter check
  filterTable();
}

function initializeProfile() {
  const tabs = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabs.forEach(tab => {
      tab.addEventListener('click', () => {
        // Remove active class from all tabs and contents
        tabs.forEach(t => t.classList.remove('active'));
        tabContents.forEach(c => c.classList.remove('active'));
        
        // Add active class to clicked tab and corresponding content
        tab.classList.add('active');
        const tabId = tab.getAttribute('data-tab');
        document.getElementById(`${tabId}-tab`).classList.add('active');
      });
    });
}

