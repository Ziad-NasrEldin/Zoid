import { ChevronDown, Check } from "lucide-react";
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

export type GlobalDropdownOption = {
  value: string;
  label: string;
  meta?: string;
  disabled?: boolean;
};

type GlobalDropdownProps = {
  id?: string;
  label: string;
  options: GlobalDropdownOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: string;
  size?: "compact" | "regular";
};

export function GlobalDropdown({ id, label, options, value, onChange, disabled = false, className = "", size = "regular" }: GlobalDropdownProps) {
  const generatedId = useId();
  const dropdownId = id ?? `zoid-dropdown-${generatedId}`;
  const menuId = `${dropdownId}-menu`;
  const rootRef = useRef<HTMLDivElement>(null);
  const optionRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [focusedIndex, setFocusedIndex] = useState(0);
  const selectedIndex = Math.max(0, options.findIndex((option) => option.value === value));
  const selectedOption = options[selectedIndex] ?? options[0];
  const canOpen = !disabled && options.some((option) => !option.disabled);

  useEffect(() => {
    if (!isOpen) return undefined;
    function handlePointerDown(event: PointerEvent) {
      if (!rootRef.current?.contains(event.target as Node)) setIsOpen(false);
    }
    window.addEventListener("pointerdown", handlePointerDown);
    return () => window.removeEventListener("pointerdown", handlePointerDown);
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    optionRefs.current[focusedIndex]?.focus();
  }, [focusedIndex, isOpen]);

  function firstEnabledIndex() {
    const index = options.findIndex((option) => !option.disabled);
    return index < 0 ? 0 : index;
  }

  function nextEnabledIndex(startIndex: number, direction: 1 | -1) {
    if (options.length === 0) return 0;
    for (let step = 1; step <= options.length; step += 1) {
      const candidateIndex = (startIndex + direction * step + options.length) % options.length;
      if (!options[candidateIndex].disabled) return candidateIndex;
    }
    return startIndex;
  }

  function openMenu(nextFocusedIndex = selectedIndex >= 0 && !options[selectedIndex]?.disabled ? selectedIndex : firstEnabledIndex()) {
    if (!canOpen) return;
    setFocusedIndex(nextFocusedIndex);
    setIsOpen(true);
  }

  function selectOption(option: GlobalDropdownOption) {
    if (option.disabled || disabled) return;
    onChange(option.value);
    setIsOpen(false);
  }

  function handleTriggerKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      openMenu(nextEnabledIndex(selectedIndex, 1));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      openMenu(nextEnabledIndex(selectedIndex, -1));
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      openMenu();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setIsOpen(false);
    }
  }

  function handleOptionKeyDown(event: KeyboardEvent<HTMLButtonElement>, index: number, option: GlobalDropdownOption) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setFocusedIndex(nextEnabledIndex(index, 1));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setFocusedIndex(nextEnabledIndex(index, -1));
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      setFocusedIndex(firstEnabledIndex());
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      setFocusedIndex(nextEnabledIndex(0, -1));
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setIsOpen(false);
      window.requestAnimationFrame(() => document.getElementById(dropdownId)?.focus());
      return;
    }
    if (event.key === "Tab") {
      setIsOpen(false);
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      selectOption(option);
      window.requestAnimationFrame(() => document.getElementById(dropdownId)?.focus());
    }
  }

  return (
    <div className={`zoid-dropdown zoid-dropdown--${size} ${className}`.trim()} ref={rootRef} data-global-dropdown="true">
      <button
        aria-controls={menuId}
        aria-expanded={isOpen}
        aria-haspopup="menu"
        aria-label={label}
        className="zoid-dropdown-trigger"
        disabled={!canOpen}
        id={dropdownId}
        onClick={() => isOpen ? setIsOpen(false) : openMenu()}
        onKeyDown={handleTriggerKeyDown}
        type="button"
      >
        <span className="zoid-dropdown-value">{selectedOption?.label ?? label}</span>
        {selectedOption?.meta ? <span className="zoid-dropdown-meta">{selectedOption.meta}</span> : null}
        <ChevronDown aria-hidden="true" className="zoid-dropdown-chevron" size={14} strokeWidth={2.7} />
      </button>
      {isOpen ? (
        <div aria-labelledby={dropdownId} className="zoid-dropdown-menu" id={menuId} role="menu">
          {options.map((option, index) => {
            const isSelected = option.value === value;
            return (
              <button
                aria-checked={isSelected}
                aria-disabled={option.disabled || undefined}
                className={isSelected ? "zoid-dropdown-option is-selected" : "zoid-dropdown-option"}
                disabled={option.disabled}
                key={option.value}
                onClick={() => selectOption(option)}
                onFocus={() => setFocusedIndex(index)}
                onKeyDown={(event) => handleOptionKeyDown(event, index, option)}
                ref={(element) => { optionRefs.current[index] = element; }}
                role="menuitemradio"
                type="button"
              >
                <span>
                  <strong>{option.label}</strong>
                  {option.meta ? <small>{option.meta}</small> : null}
                </span>
                {isSelected ? <Check aria-hidden="true" size={13} strokeWidth={3} /> : null}
              </button>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
