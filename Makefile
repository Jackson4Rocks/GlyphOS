PROFILE := profile
WORK := work
OUT := out

.PHONY: build clean run validate

build:
	mkdir -p $(WORK) $(OUT)
	mkarchiso -v -w $(WORK) -o $(OUT) $(PROFILE)

clean:
	rm -rf $(WORK) $(OUT)

validate:
	./scripts/validate-profile.sh

run: build
	run_archiso -u -i $$(ls -t $(OUT)/*.iso | head -n1)
